# yt-dlp-fe

A simple cross-platform desktop app to download YouTube audio as MP3 using [yt-dlp](https://github.com/yt-dlp/yt-dlp).

## Features

- Download YouTube videos as MP3 audio
- Simple minimalist UI
- Cross-platform (Linux, macOS, Windows)

## Requirements

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) installed on your system
- For development: Node.js, Rust

### Install yt-dlp

**macOS:**
```bash
brew install yt-dlp
```

**Linux:**
```bash
pip install yt-dlp
# or
sudo dnf install yt-dlp
```

**Windows:**
```powershell
pip install yt-dlp
```

## Development

### Prerequisites

- Node.js 18+
- Rust 1.70+

**Linux (Fedora/RHEL):**
```bash
sudo dnf install -y gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
```

**macOS:**
```bash
brew install gtk+3 webkit2gtk-4.1
```

### Setup

```bash
npm install
```

### Run in development mode

```bash
npm run tauri dev
```

### Build for production

```bash
npm run tauri build
```

The built application will be in:
- Linux: `src-tauri/target/release/yt-dlp-fe`
- Debian: `src-tauri/target/release/bundle/deb/`
- macOS: `src-tauri/target/release/bundle/macos/`

## Usage

1. Enter a YouTube URL in the input field
2. Click "Download" to convert to MP3
3. The file will be saved to your Downloads folder

## License

MIT
