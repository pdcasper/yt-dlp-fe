use std::path::Path;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;

struct AppState {
    download_dir: Mutex<Option<PathBuf>>,
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

    let output = app
        .shell()
        .command("yt-dlp")
        .args([
            "-x",
            "--audio-format",
            "mp3",
            "--output",
            &output_template,
            &url,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

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
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            download_mp3,
            get_download_dir,
            set_download_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
