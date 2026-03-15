const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const urlInput = document.querySelector("#url-input");
const downloadBtn = document.querySelector("#download-btn");
const progressContainer = document.querySelector("#progress-container");
const progressFill = document.querySelector("#progress-fill");
const statusText = document.querySelector("#status-text");
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
  urlInput.disabled = true;
  
  progressContainer.classList.remove("hidden");
  result.classList.add("hidden");
  
  progressFill.style.width = "30%";
  statusText.textContent = "Starting download...";

  try {
    const response = await invoke("download_mp3", { url });
    
    progressFill.style.width = "100%";
    statusText.textContent = "Complete!";
    showResult(response, "success");
  } catch (error) {
    showResult(error, "error");
  } finally {
    isDownloading = false;
    downloadBtn.disabled = false;
    urlInput.disabled = false;
  }
}

function showResult(message, type) {
  resultText.textContent = message;
  result.classList.remove("hidden", "success", "error");
  result.classList.add(type);
  progressContainer.classList.add("hidden");
}

listen("download-started", () => {
  progressFill.style.width = "50%";
  statusText.textContent = "Downloading...";
});

listen("download-progress", (event) => {
  const { percent, status } = event.payload;
  progressFill.style.width = `${percent}%`;
  statusText.textContent = status;
});

listen("download-complete", () => {
  progressFill.style.width = "100%";
  statusText.textContent = "Complete!";
});

listen("download-error", (event) => {
  showResult(event.payload, "error");
});

window.addEventListener("DOMContentLoaded", () => {
  downloadBtn.addEventListener("click", startDownload);
  
  urlInput.addEventListener("keypress", (e) => {
    if (e.key === "Enter" && !isDownloading) {
      startDownload();
    }
  });
});
