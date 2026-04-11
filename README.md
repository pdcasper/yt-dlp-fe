# yt-dlp-fe

A simple cross-platform desktop app to download YouTube audio as MP3.

## Features

- Download YouTube videos as MP3 audio
- Support for playlists and single videos
- Simple minimalist UI
- Cross-platform (Linux, macOS, Windows)
- **No external dependencies required!** yt-dlp and ffmpeg are bundled with the app

## Downloads

Pre-built binaries are available on the [Releases](https://github.com/pdcasper/yt-dlp-fe/releases) page.

| Platform | File |
|----------|------|
| Windows | `yt-dlp-fe_x.x.x_x64-setup.exe` or `.msi` |
| macOS (Intel) | `yt-dlp-fe_x.x.x_x64.dmg` |
| macOS (Apple Silicon) | `yt-dlp-fe_x.x.x_aarch64.dmg` |
| Linux | `yt-dlp-fe_x.x.x_amd64.deb` |

## Development

### Prerequisites

- Node.js 18+
- Rust 1.70+

**macOS:**
```bash
brew install gtk+3
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

**Linux (Fedora/RHEL):**
```bash
sudo dnf install -y gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
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
- Linux: `src-tauri/target/release/bundle/deb/`
- macOS: `src-tauri/target/release/bundle/dmg/`
- Windows: `src-tauri/target/release/bundle/nsis/`

### Releasing

Create a new tag to trigger the CI build and release:

```bash
git tag v1.0.0
git push origin v1.0.0
```

## Usage

1. Enter a YouTube URL in the input field
2. Click "Download" to download the audio
3. The file will be saved to your Downloads folder

---

## Licencia

MIT