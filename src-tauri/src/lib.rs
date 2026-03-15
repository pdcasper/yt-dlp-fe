use std::path::Path;

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tauri::{AppHandle, Emitter, Manager};

struct AppState {
    download_dir: Mutex<Option<PathBuf>>,
    current_pid: Mutex<Option<u32>>,
    cancel_flag: Arc<AtomicBool>,
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

#[tauri::command]
async fn download_mp3(app: AppHandle, url: String) -> Result<String, String> {
    let output_dir = get_effective_download_dir(&app);

    let output_template = output_dir
        .join("%(title)s.%(ext)s")
        .to_string_lossy()
        .to_string();

    let _ = app.emit("download-started", ());

    let cancel_flag = app.state::<AppState>().cancel_flag.clone();
    cancel_flag.store(false, std::sync::atomic::Ordering::SeqCst);

    #[cfg(unix)]
    let child = {
        use std::os::unix::process::CommandExt;
        let mut cmd = std::process::Command::new("yt-dlp");
        cmd.args([
            "-x",
            "--audio-format",
            "mp3",
            "--output",
            &output_template,
            &url,
        ]);
        cmd.process_group(0);
        cmd.spawn()
            .map_err(|e| format!("Failed to start yt-dlp: {}", e))?
    };

    #[cfg(not(unix))]
    let child = {
        std::process::Command::new("yt-dlp")
            .args([
                "-x",
                "--audio-format",
                "mp3",
                "--output",
                &output_template,
                &url,
            ])
            .spawn()
            .map_err(|e| format!("Failed to start yt-dlp: {}", e))?
    };

    let pid = child.id();
    {
        let state = app.state::<AppState>();
        *state.current_pid.lock().unwrap() = Some(pid);
    }

    let output = child.wait_with_output().map_err(|e| format!("Failed to wait: {}", e))?;

    let was_cancelled = cancel_flag.load(std::sync::atomic::Ordering::SeqCst);

    {
        let state = app.state::<AppState>();
        *state.current_pid.lock().unwrap() = None;
    }

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
                current_pid: Mutex::new(None),
                cancel_flag: Arc::new(AtomicBool::new(false)),
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