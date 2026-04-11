# yt-dlp-fe

A simple cross-platform desktop app to download YouTube audio as MP3.

## Features

- Download YouTube videos as MP3 audio
- Support for playlists and single videos
- Simple minimalist UI
- Cross-platform (Linux, macOS, Windows)

## Requirements

- No external dependencies required (yt-dlp is bundled with the app)
- For development: Node.js, Rust, yt-dlp

## Development

### Prerequisites

- Node.js 18+
- Rust 1.70+
- yt-dlp
- ffmpeg

**macOS:**
```bash
brew install yt-dlp ffmpeg
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt install yt-dlp ffmpeg
```

**Linux (Fedora/RHEL):**
```bash
sudo dnf install yt-dlp ffmpeg
```

**Windows:**
```powershell
pip install yt-dlp
winget install ffmpeg
```

### System Libraries

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

### Releasing

Create a new tag to trigger the CI build and release:

```bash
git tag v1.0.0
git push origin v1.0.0
```

## Downloads

Pre-built binaries are available on the [Releases](https://github.com/pdcasper/yt-dlp-fe/releases) page.

| Platform | File |
|----------|------|
| Windows | `yt-dlp-fe_x.x.x_x64-setup.exe` or `.msi` |
| macOS (Intel) | `yt-dlp-fe_x.x.x_x64.dmg` |
| macOS (Apple Silicon) | `yt-dlp-fe_x.x.x_aarch64.dmg` |
| Linux | `yt-dlp-fe_x.x.x_amd64.deb` |

## Usage

1. Enter a YouTube URL in the input field
2. Click "Download" to download the audio
3. The file will be saved to your Downloads folder

---

# yt-dlp-fe (Espanol)

Una aplicacion de escritorio multiplataforma para descargar audio de YouTube como MP3.

## Caracteristicas

- Descargar videos de YouTube como audio MP3
- Soporte para listas de reproduccion y videos individuales
- Interfaz minimalista y sencilla
- Multiplataforma (Linux, macOS, Windows)

## Requisitos

- Sin dependencias externas (yt-dlp viene incluido)
- Para desarrollo: Node.js, Rust, yt-dlp, ffmpeg

## Desarrollo

### Requisitos previos

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

### Configuracion

```bash
npm install
```

### Ejecutar en modo desarrollo

```bash
npm run tauri dev
```

### Compilar para produccion

```bash
npm run tauri build
```

La aplicacion compilada estara en:
- Linux: `src-tauri/target/release/yt-dlp-fe`
- Debian: `src-tauri/target/release/bundle/deb/`
- macOS: `src-tauri/target/release/bundle/macos/`

### Crear un lanzamiento

Crea una nueva etiqueta para activar la compilacion y publicacion:

```bash
git tag v1.0.0
git push origin v1.0.0
```

## Descargas

Los binarios pre-construidos estan disponibles en la pagina de [Lanzamientos](https://github.com/pdcasper/yt-dlp-fe/releases).

| Plataforma | Archivo |
|------------|---------|
| Windows | `yt-dlp-fe_x.x.x_x64-setup.exe` o `.msi` |
| macOS (Intel) | `yt-dlp-fe_x.x.x_x64.dmg` |
| macOS (Apple Silicon) | `yt-dlp-fe_x.x.x_aarch64.dmg` |
| Linux | `yt-dlp-fe_x.x.x_amd64.deb` |

## Uso

1. Ingresa una URL de YouTube en el campo de texto
2. Haz clic en "Descargar" para descargar el audio
3. El archivo se guardara en tu carpeta de Descargas

## Licencia

MIT
