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

fn convert_to_mp3(input_path: &Path, output_path: &Path, app: &AppHandle, cancel_flag: &Arc<AtomicBool>, progress: &Arc<AtomicU64>) -> Result<(), String> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::probe::Hint;
    use mp3lame_encoder::{Builder, DualPcm, FlushNoGap};

    let file = fs::File::open(input_path)
        .map_err(|e| format!("Failed to open downloaded file: {}", e))?;
    
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let mut hint = Hint::new();
    hint.with_extension("m4a");

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| format!("Unsupported format: {}", e))?;

    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or("No default track found")?
        .clone();

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    let mut builder = Builder::new()
        .ok_or("Failed to create MP3 encoder")?;
    builder.set_num_channels(channels as u8)
        .map_err(|e| format!("set_num_channels error: {}", e))?;
    builder.set_sample_rate(sample_rate as u32)
        .map_err(|e| format!("set_sample_rate error: {}", e))?;
    builder.set_brate(mp3lame_encoder::Bitrate::Kbps192)
        .map_err(|e| format!("set_brate error: {}", e))?;
    builder.set_quality(mp3lame_encoder::Quality::Best)
        .map_err(|e| format!("set_quality error: {}", e))?;
    let mut mp3_encoder = builder.build()
        .map_err(|e| format!("Failed to create MP3 encoder: {}", e))?;

    let mut mp3_output = Vec::new();

    loop {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("Conversion cancelled".to_string());
        }

        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e)) 
                if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(format!("Read error: {}", e)),
        };

        if packet.track_id() != track.id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let mut sample_buf = SampleBuffer::<i16>::new(decoded.capacity() as u64, spec);
                sample_buf.copy_interleaved_ref(decoded);

                let samples = sample_buf.samples();
                let len = samples.len();
                let left = &samples[..len / 2];
                let right = if len > channels as usize {
                    &samples[len / 2..]
                } else {
                    left
                };

                let pcm = DualPcm { left, right };
                let mut encoded = Vec::new();
                mp3_encoder.encode_to_vec(pcm, &mut encoded)
                    .map_err(|e| format!("Encoding error: {}", e))?;
                mp3_output.extend(encoded);

                let percent = progress.load(std::sync::atomic::Ordering::SeqCst);
                let new_percent = (percent + 5).min(99);
                progress.store(new_percent, std::sync::atomic::Ordering::SeqCst);
                let _ = app.emit("download-progress", new_percent);
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("Decode error: {}", e)),
        }
    }

    let mut final_samples = Vec::new();
    mp3_encoder.flush_to_vec::<FlushNoGap>(&mut final_samples)
        .map_err(|e| format!("Flush error: {}", e))?;
    mp3_output.extend(final_samples);

    fs::write(output_path, &mp3_output)
        .map_err(|e| format!("Failed to write MP3: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn download_mp3(app: AppHandle, url: String) -> Result<String, String> {
    let output_dir = get_effective_download_dir(&app);

    let _ = app.emit("download-started", ());

    let cancel_flag = app.state::<AppState>().cancel_flag.clone();
    cancel_flag.store(false, std::sync::atomic::Ordering::SeqCst);

    let progress = app.state::<AppState>().progress.clone();

    let video_url = if url.contains("&list=") || url.contains("playlist") {
        if let Some(video_id) = rusty_ytdl::get_video_id(&url) {
            format!("https://www.youtube.com/watch?v={}", video_id)
        } else {
            return Err("Could not extract video ID from URL".to_string());
        }
    } else {
        url
    };

    let video_info = rusty_ytdl::Video::new(&video_url)
        .map_err(|e| format!("Failed to create video: {}", e))?
        .get_info()
        .await
        .map_err(|e| format!("Failed to get video info: {}", e))?;

    let title = video_info.video_details.title;
    let sanitized_title: String = title
        .chars()
        .filter(|c: &char| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect::<String>()
        .trim()
        .to_string();

    let output_path = output_dir.join(format!("{}.mp3", sanitized_title));

    let temp_dir = std::env::temp_dir();
    let temp_input = temp_dir.join(format!("{}.m4a", sanitized_title));

    let video_options = rusty_ytdl::VideoOptions {
        quality: rusty_ytdl::VideoQuality::Highest,
        filter: rusty_ytdl::VideoSearchOptions::Audio,
        ..Default::default()
    };

    let video = rusty_ytdl::Video::new_with_options(&video_url, video_options)
        .map_err(|e| format!("Failed to create video: {}", e))?;

    let stream = video
        .stream()
        .await
        .map_err(|e| format!("Failed to get stream: {}", e))?;

    let mut file = fs::File::create(&temp_input)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    use std::io::Write;
    let mut downloaded: u64 = 0;
    let total_size = stream.content_length() as u64;

    loop {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            drop(file);
            fs::remove_file(&temp_input).ok();
            return Err("Download cancelled".to_string());
        }

        match stream.chunk().await {
            Ok(Some(chunk)) => {
                file.write_all(&chunk)
                    .map_err(|e| format!("Write error: {}", e))?;
                downloaded += chunk.len() as u64;

                if total_size > 0 {
                    let percent = ((downloaded * 100) / total_size).min(49);
                    progress.store(percent, std::sync::atomic::Ordering::SeqCst);
                    let _ = app.emit("download-progress", percent);
                }
            }
            Ok(None) => break,
            Err(e) => {
                drop(file);
                fs::remove_file(&temp_input).ok();
                return Err(format!("Download error: {}", e));
            }
        }
    }

    drop(file);

    progress.store(50, std::sync::atomic::Ordering::SeqCst);
    let _ = app.emit("download-progress", 50u64);

    convert_to_mp3(&temp_input, &output_path, &app, &cancel_flag, &progress)?;

    fs::remove_file(&temp_input).ok();

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
