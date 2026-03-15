use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;

#[tauri::command]
async fn download_mp3(app: AppHandle, url: String) -> Result<String, String> {
    let downloads_dir =
        dirs::download_dir().ok_or_else(|| "Could not find downloads directory".to_string())?;

    let output_template = downloads_dir
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
        Ok("Download complete! Check your Downloads folder.".to_string())
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
        .invoke_handler(tauri::generate_handler![download_mp3])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
