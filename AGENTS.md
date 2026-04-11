# yt-dlp-fe 🎵▶️

```
   __      __                _           _ 
   \ \    / /               | |         | |
    \ \  / /__ _ __ ___  ___| | ___  ___| |
     \ \/ / _ \ '__/ _ \/ __| |/ _ \/ __| |
      \  /  __/ | |  __/ (__| |  __/\__ \_|
       \/ \___|_|  \___|\___|_|\___||___(_)
                                             
   __  __                                     
  |  \/  |                                   
  | \  / | ___ _ __  ___  ___  _ __ ___  ___ 
  | |\/| |/ _ \ '_ \/ __|/ _ \| '__/ _ \/ __|
  | |  | |  __/ | | \__ \ (_) | | |  __/\__ \
  |_|  |_|\___|_| |_|___/\___/|_|  \___||___/
                                             
```

This document provides guidelines for agents working on the yt-dlp-fe codebase.

## Project Overview

yt-dlp-fe is a Tauri 2.0 desktop application that downloads YouTube audio as MP3 using yt-dlp. The project consists of:
- **Frontend**: Vanilla JavaScript in `src/` directory
- **Backend**: Rust in `src-tauri/src/` directory
- **Build System**: Tauri CLI with npm scripts

## Build, Lint, and Test Commands

### Frontend Commands (from project root)

```bash
# Development - runs both frontend and Tauri backend
npm run tauri dev

# Production build
npm run tauri build

# Run Tauri CLI directly
npm run tauri <command>
```

### Rust Backend Commands (from src-tauri directory)

```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run the application
cargo run

# Run a single test
cargo test <test_name>
cargo test --test <test_file_name>  # Run tests from a specific test file

# Run all tests
cargo test

# Check code (faster than build, for linting)
cargo check

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy (Rust linter)
cargo clippy

# Update dependencies
cargo update
```

### Quick Reference

| Task | Command |
|------|---------|
| Dev server | `npm run tauri dev` |
| Production build | `npm run tauri build` |
| Single Rust test | `cargo test test_name_here` |
| All Rust tests | `cargo test` |
| Check for errors | `cargo check` |
| Format Rust code | `cargo fmt` |
| Lint Rust code | `cargo clippy` |

## Code Style Guidelines

### Rust (Backend)

#### General Conventions
- Follow standard Rust idioms and conventions
- Use `cargo fmt` for formatting (enforces 4-space indentation)
- Use `cargo clippy` to catch common mistakes

#### Naming Conventions
- **Functions/variables**: `snake_case` (e.g., `download_mp3`, `output_template`)
- **Types/Enums**: `PascalCase` (e.g., `AppHandle`, `Result`)
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

#### Error Handling
- Use `Result<T, String>` for command return types in Tauri
- Provide meaningful error messages with context
- Convert errors to strings with `.to_string()` or `format!()` macros
- Example:
  ```rust
  .map_err(|e| format!("Failed to run yt-dlp: {}", e))?
  ```

#### Tauri Commands
- Mark async functions with `#[tauri::command]`
- Use `AppHandle` for emitting events to frontend
- Return `Result<T, String>` for error propagation
- Example:
  ```rust
  #[tauri::command]
  async fn command_name(app: AppHandle, param: String) -> Result<String, String> {
      // implementation
  }
  ```

#### Imports
- Use absolute imports from crate root when possible
- Group standard library, external crates, and local modules
- Example:
  ```rust
  use std::process::Command;
  use tauri::{AppHandle, Emitter};
  ```

#### Dependencies
- Keep dependencies minimal
- Use `serde` with `derive` feature for serialization
- Use `dirs` for platform-specific directory paths

### JavaScript/HTML/CSS (Frontend)

#### General Conventions
- Use vanilla JavaScript (no frameworks)
- Keep code simple and readable
- Use semantic HTML

#### Naming Conventions
- **Variables/functions**: `camelCase` (e.g., `downloadButton`, `handleClick`)
- **CSS classes**: `kebab-case` (e.g., `.download-button`, `.input-field`)
- **Constants**: `UPPER_SNAKE_CASE` or `camelCase` with prefix

#### Event Handling
- Use addEventListener for DOM events
- Handle errors gracefully with user feedback
- Emit events to Tauri backend using `window.__TAURI__.invoke`

### File Organization

```
yt-dlp-fe/
├── src/                      # Frontend (served as static files)
│   ├── index.html           # Main HTML entry
│   ├── main.js              # JavaScript entry
│   ├── styles.css           # CSS styles
│   └── assets/              # Static assets
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs           # Tauri commands and app logic
│   │   ├── main.rs          # Entry point (calls lib::run)
│   │   └── build.rs         # Build script
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── package.json             # npm configuration
└── README.md
```

### Tauri-Specific Guidelines

#### Configuration (tauri.conf.json)
- `frontendDist`: Path to frontend static files (default: `../src`)
- `app.windows`: Window configuration (size, title, etc.)
- `bundle.targets`: Build targets (deb, rpm, etc.)

#### Security
- Keep CSP settings appropriate for your needs
- Be cautious with `withGlobalTauri: true` (exposes Tauri APIs globally)

#### Platform-Specific
- Use `dirs` crate for cross-platform directory access
- Test on multiple platforms (Linux, macOS, Windows) before releases

## Common Patterns

### Calling Rust from JavaScript

```javascript
// Invoke a Tauri command
const result = await window.__TAURI__.core.invoke('command_name', { param: 'value' });

// Listen for events
const { listen } = window.__TAURI__.event;
listen('event-name', (event) => {
    console.log('Event received:', event.payload);
});
```

### Running External Commands

```rust
use std::process::Command;

let output = Command::new("yt-dlp")
    .args(["-x", "--audio-format", "mp3", "--output", &template, &url])
    .output()
    .map_err(|e| format!("Failed to run command: {}", e))?;

if output.status.success() {
    // handle success
} else {
    // handle failure - get error from stderr
    let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
}
```

## Development Workflow

1. **Setup**: Run `npm install` to install dependencies
2. **Develop**: Use `npm run tauri dev` for live development
3. **Test**: Run `cargo test` for Rust tests
4. **Build**: Use `npm run tauri build` for production builds
5. **Format**: Run `cargo fmt` before committing

## Requirements

- Node.js 18+
- Rust 1.70+
- yt-dlp CLI installed on system
