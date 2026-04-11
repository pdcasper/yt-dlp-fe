const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { open } = window.__TAURI__.dialog;

const urlInput = document.querySelector("#url-input");
const downloadBtn = document.querySelector("#download-btn");
const cancelBtn = document.querySelector("#cancel-btn");
const folderBtn = document.querySelector("#folder-btn");
const progressContainer = document.querySelector("#progress-container");
const progressFill = document.querySelector("#progress-fill");
const statusText = document.querySelector("#status-text");
const songTitle = document.querySelector("#song-title");
const result = document.querySelector("#result");
const resultText = document.querySelector("#result-text");

let isDownloading = false;

async function startDownload() {
  const url = urlInput.value.trim();
  
  if (!url) {
    showResult("Please enter a YouTube URL", "error");
    return;
  }

  if (!url.includes("youtube.com") && !url.includes("youtu.be")) {
    showResult("Please enter a valid YouTube URL", "error");
    return;
  }

  isDownloading = true;
  downloadBtn.disabled = true;
  cancelBtn.disabled = false;
  cancelBtn.classList.remove("hidden");
  urlInput.disabled = true;
  
  progressContainer.classList.remove("hidden");
  result.classList.add("hidden");
  
  progressFill.style.width = "0%";
  statusText.textContent = "Starting...";
  songTitle.textContent = "";

  try {
    const response = await invoke("download_mp3", { url });
    
    progressFill.style.width = "100%";
    statusText.textContent = "Complete!";
    songTitle.textContent = "";
    showResult(response, "success");
  } catch (error) {
    if (!error.includes("cancelled")) {
      showResult(error, "error");
    }
  } finally {
    isDownloading = false;
    downloadBtn.disabled = false;
    cancelBtn.disabled = true;
    cancelBtn.classList.add("hidden");
    urlInput.disabled = false;
  }
}

async function cancelDownload() {
  try {
    await invoke("cancel_download");
    statusText.textContent = "Cancelled";
    songTitle.textContent = "";
    isDownloading = false;
    downloadBtn.disabled = false;
    cancelBtn.disabled = true;
    cancelBtn.classList.add("hidden");
    urlInput.disabled = false;
  } catch (error) {
    console.error("Failed to cancel:", error);
  }
}

async function selectFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
  });

  if (selected) {
    try {
      await invoke("set_download_dir", { path: selected });
      folderBtn.title = selected;
      showResult(`Download folder set to: ${selected}`, "success");
    } catch (error) {
      showResult(`Error: ${error}`, "error");
    }
  }
}

function showResult(message, type) {
  resultText.textContent = message;
  result.classList.remove("hidden", "success", "error");
  result.classList.add(type);
  progressContainer.classList.add("hidden");
}

async function initFolder() {
  try {
    const dir = await invoke("get_download_dir");
    folderBtn.title = dir;
  } catch (e) {
    console.error("Failed to get download dir:", e);
  }
}

listen("download-started", () => {
  progressFill.style.width = "0%";
  statusText.textContent = "Starting...";
  songTitle.textContent = "";
});

listen("download-progress", (event) => {
  const { title, percent } = event.payload;
  
  if (title) {
    songTitle.textContent = title;
  }
  
  if (percent > 0 && percent <= 100) {
    statusText.textContent = `Downloading ${percent}%`;
    progressFill.style.width = `${percent}%`;
  } else if (title) {
    statusText.textContent = title;
  }
});

listen("download-complete", () => {
  progressFill.style.width = "100%";
  statusText.textContent = "Complete!";
  songTitle.textContent = "";
});

listen("download-error", (event) => {
  showResult(event.payload, "error");
});

listen("download-cancelled", () => {
  statusText.textContent = "Cancelled";
  songTitle.textContent = "";
  isDownloading = false;
  downloadBtn.disabled = false;
  cancelBtn.disabled = true;
  cancelBtn.classList.add("hidden");
  urlInput.disabled = false;
});

window.addEventListener("DOMContentLoaded", () => {
  downloadBtn.addEventListener("click", startDownload);
  cancelBtn.addEventListener("click", cancelDownload);
  folderBtn.addEventListener("click", selectFolder);
  
  urlInput.addEventListener("keypress", (e) => {
    if (e.key === "Enter" && !isDownloading) {
      startDownload();
    }
  });

  initFolder();
});
