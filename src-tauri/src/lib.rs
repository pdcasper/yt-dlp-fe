use std::path::Path;

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64}};
use tauri::{AppHandle, Emitter, Manager};

struct AppState {
    download_dir: Mutex<Option<PathBuf>>,
    cancel_flag: Arc<AtomicBool>,
    progress: Arc<AtomicU64>,
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

    let _ = app.emit("download-started", ());

    let cancel_flag = app.state::<AppState>().cancel_flag.clone();
    cancel_flag.store(false, std::sync::atomic::Ordering::SeqCst);

    let progress = app.state::<AppState>().progress.clone();

    let video_info = rusty_ytdl::Video::new(&url)
        .map_err(|e| format!("Failed to create video: {}", e))?
        .get_info()
        .await
        .map_err(|e| format!("Failed to get video info: {}", e))?;

    let title = video_info
        .video_details
        .title;

    let sanitized_title: String = title
        .chars()
        .filter(|c: &char| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect::<String>()
        .trim()
        .to_string();

    let output_path = output_dir.join(format!("{}.mp3", sanitized_title));

    let video_options = rusty_ytdl::VideoOptions {
        quality: rusty_ytdl::VideoQuality::Highest,
        filter: rusty_ytdl::VideoSearchOptions::Audio,
        ..Default::default()
    };

    let video = rusty_ytdl::Video::new_with_options(&url, video_options)
        .map_err(|e| format!("Failed to create video: {}", e))?;

    let stream = video
        .stream()
        .await
        .map_err(|e| format!("Failed to get stream: {}", e))?;

    let mut file = fs::File::create(&output_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut downloaded: u64 = 0;
    let total_size = stream.content_length() as u64;

    loop {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            drop(file);
            fs::remove_file(&output_path).ok();
            return Err("Download cancelled".to_string());
        }

        match stream.chunk().await {
            Ok(Some(chunk)) => {
                use std::io::Write;
                file.write_all(&chunk)
                    .map_err(|e| format!("Write error: {}", e))?;
                downloaded += chunk.len() as u64;

                if total_size > 0 {
                    let percent = (downloaded * 100) / total_size;
                    progress.store(percent, std::sync::atomic::Ordering::SeqCst);
                    let _ = app.emit("download-progress", percent);
                }
            }
            Ok(None) => break,
            Err(e) => {
                drop(file);
                fs::remove_file(&output_path).ok();
                return Err(format!("Download error: {}", e));
            }
        }
    }

    let _ = app.emit("download-complete", ());

    Ok(format!(
        "Download complete! Saved to {}",
        output_path.display()
    ))
}

#[tauri::command]
async fn cancel_download(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    state.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    let _ = app.emit("download-cancelled", ());
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
                progress: Arc::new(AtomicU64::new(0)),
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                state.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
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
