use std::path::Path;
use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tauri::{AppHandle, Emitter, Manager};

struct AppState {
    download_dir: Mutex<Option<PathBuf>>,
    cancel_flag: Arc<AtomicBool>,
    current_pid: Mutex<Option<u32>>,
}

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    current: u32,
    total: u32,
    title: String,
    percent: u32,
}

fn get_config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("download_dir.json")
}

fn load_download_dir(app: &AppHandle) -> Option<PathBuf> {
    let path = get_config_path(app);
    if path.exists() {
        let content = fs::read_to_string(&path).ok()?;
        let dir: String = serde_json::from_str(&content).ok()?;
        let pb = PathBuf::from(&dir);
        if pb.is_dir() {
            return Some(pb);
        }
    }
    None
}

fn save_download_dir(app: &AppHandle, dir: &Path) -> Result<(), String> {
    let path = get_config_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content =
        serde_json::to_string(&dir.to_string_lossy().to_string()).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    app.state::<AppState>()
        .download_dir
        .lock()
        .unwrap()
        .replace(dir.to_path_buf());
    Ok(())
}

fn get_effective_download_dir(app: &AppHandle) -> PathBuf {
    let state = app.state::<AppState>();
    if let Some(ref dir) = *state.download_dir.lock().unwrap() {
        return dir.clone();
    }
    dirs::download_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn get_yt_dlp_path() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let bundled = exe_dir.join("yt-dlp");
            if bundled.exists() {
                return bundled;
            }
            #[cfg(target_os = "macos")]
            {
                let resources = exe_dir.join("yt-dlp-fe.app").join("Contents").join("Resources");
                let bundled = resources.join("yt-dlp");
                if bundled.exists() {
                    return bundled;
                }
            }
        }
    }
    PathBuf::from("yt-dlp")
}

#[tauri::command]
fn get_download_dir(app: AppHandle) -> String {
    get_effective_download_dir(&app)
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
fn set_download_dir(app: AppHandle, path: String) -> Result<(), String> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err("Invalid directory path".to_string());
    }
    save_download_dir(&app, &dir)
}

fn parse_yt_dlp_progress(line: &str) -> Option<(String, u32)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if line.contains("[download]") {
        if let Ok(re) = regex::Regex::new(r"(\d+\.?\d*)%") {
            if let Some(caps) = re.captures(line) {
                if let Some(m) = caps.get(1) {
                    if let Ok(pct) = m.as_str().parse::<f64>() {
                        let title = if let Ok(title_re) = regex::Regex::new(r"of\s+(.+?)(?:\s+at|\s+of\s+$)") {
                            title_re.captures(line).and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| "Downloading".to_string())
                        } else {
                            "Downloading".to_string()
                        };
                        return Some((title, pct as u32));
                    }
                }
            }
        }
    }
    
    None
}

fn run_yt_dlp_with_progress(
    yt_dlp_path: &Path,
    url: &str,
    output_template: &str,
    cancel_flag: Arc<AtomicBool>,
    app: AppHandle,
) -> Result<std::process::Output, String> {
    let _ = app.emit("download-started", ());
    
    let mut cmd = std::process::Command::new(yt_dlp_path);
    cmd.args([
        "-x",
        "--audio-format",
        "mp3",
        "--output",
        output_template,
        url,
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    let mut child = cmd.spawn().map_err(|e| format!("Failed to start yt-dlp: {}", e))?;
    let pid = child.id();
    
    let app_clone = app.clone();
    let (tx, rx) = mpsc::channel();
    
    let stderr = child.stderr.take();
    let stdout = child.stdout.take();
    
    thread::spawn(move || {
        if let Some(stderr) = stderr {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    let _ = tx.send(Err("cancelled"));
                    return;
                }
                if let Some((title, percent)) = parse_yt_dlp_progress(&line) {
                    let progress = DownloadProgress {
                        current: 0,
                        total: 0,
                        title,
                        percent,
                    };
                    let _ = app_clone.emit("download-progress", &progress);
                }
            }
        }
        drop(stdout);
        let _ = tx.send(Ok(()));
    });

    {
        let state = app.state::<AppState>();
        *state.current_pid.lock().unwrap() = Some(pid);
    }

    let output = child.wait_with_output().map_err(|e| format!("Failed to wait: {}", e))?;

    {
        let state = app.state::<AppState>();
        *state.current_pid.lock().unwrap() = None;
    }

    let _: Result<Result<(), &str>, _> = rx.recv();

    Ok(output)
}

#[tauri::command]
async fn download_mp3(app: AppHandle, url: String) -> Result<String, String> {
    let output_dir = get_effective_download_dir(&app);

    let output_template = output_dir
        .join("%(title)s.%(ext)s")
        .to_string_lossy()
        .to_string();

    let cancel_flag = app.state::<AppState>().cancel_flag.clone();
    cancel_flag.store(false, std::sync::atomic::Ordering::SeqCst);

    let yt_dlp_path = get_yt_dlp_path();

    let output = run_yt_dlp_with_progress(
        &yt_dlp_path,
        &url,
        &output_template,
        cancel_flag.clone(),
        app.clone(),
    )?;

    let was_cancelled = cancel_flag.load(std::sync::atomic::Ordering::SeqCst);

    if was_cancelled {
        return Err("Download cancelled".to_string());
    }

    if output.status.success() {
        let _ = app.emit("download-complete", ());
        Ok(format!(
            "Download complete! Saved to {}",
            output_dir.display()
        ))
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
        let _ = app.emit("download-error", &error_msg);
        Err(error_msg)
    }
}

#[tauri::command]
async fn cancel_download(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let cancel_flag = state.cancel_flag.clone();
    cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);

    if let Some(pid) = *state.current_pid.lock().unwrap() {
        #[cfg(unix)]
        {
            std::process::Command::new("kill")
                .args(["-9", &format!("-{}", pid)])
                .spawn()
                .map_err(|e| format!("Failed to kill: {}", e))?;
        }
        #[cfg(not(unix))]
        {
            std::process::Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .spawn()
                .map_err(|e| format!("Failed to kill: {}", e))?;
        }
        let _ = app.emit("download-cancelled", ());
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = load_download_dir(app.handle());
            app.manage(AppState {
                download_dir: Mutex::new(dir),
                cancel_flag: Arc::new(AtomicBool::new(false)),
                current_pid: Mutex::new(None),
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                state.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                let pid = *state.current_pid.lock().unwrap();
                if let Some(pid) = pid {
                    #[cfg(unix)]
                    {
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &format!("-{}", pid)])
                            .spawn();
                    }
                    #[cfg(not(unix))]
                    {
                        let _ = std::process::Command::new("taskkill")
                            .args(["/F", "/T", "/PID", &pid.to_string()])
                            .spawn();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            download_mp3,
            cancel_download,
            get_download_dir,
            set_download_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
