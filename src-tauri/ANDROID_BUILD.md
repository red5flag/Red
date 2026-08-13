# Red — Android Build Guide

## Prerequisites

### System Dependencies (Linux desktop build only)
```bash
# Fedora
sudo dnf install webkit2gtk4.1-devel pango-devel libsoup3-devel gtk3-devel glib2-devel rsvg2-devel

# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev libpango1.0-dev libsoup-3.0-dev libgtk-3-dev libglib2.0-dev librsvg2-dev
```

### Android Build Prerequisites
1. **Android Studio** with:
   - Android SDK Platform 34 (or minimum 24)
   - Android NDK (Side by side)
   - CMake
2. **Rust Android targets**:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   ```
3. **Tauri CLI**:
   ```bash
   cargo install tauri-cli --version "^2"
   ```

## Build Commands

### Desktop (development)
```bash
# Start Leptos dev server + Tauri window
cargo tauri dev
```

### Android (development)
```bash
# Initialize Android project (creates src-tauri/gen/android/)
cargo tauri android init

# Build and run on connected device/emulator
cargo tauri android dev

# Generate Android APK
cargo tauri android build

# Generate Android AAB (for Play Store)
cargo tauri android build --aab
```

### Web (preserved)
```bash
# SSR web server (original)
cargo leptos watch

# CSR-only web build
cargo leptos build --features csr
```

## Architecture

```
Leptos UI (CSR/WASM)
  → Android WebView
  → Tauri 2 IPC commands/events
  → Native Rust core (farley lib + red-tauri-lib)
  → sled / native networking / crypto / compression
```

## Android Permissions

Configured in `src-tauri/gen/android.properties`:
- `INTERNET` — network access for API calls
- `ACCESS_NETWORK_STATE` — connection state detection
- `POST_NOTIFICATIONS` — push notifications (Android 13+)

## Android Lifecycle

Tauri 2 handles WebView lifecycle automatically. Key considerations:
- The WebView is destroyed/recreated on configuration changes (rotation)
- Leptos state is lost on WebView recreation — use Tauri commands to persist state in sled
- Background networking requires a foreground service for long-running operations
- Use Tauri events to push updates from native to WebView

## Minimum SDK

- **minSdkVersion**: 24 (Android 7.0)
- **targetSdkVersion**: 34 (Android 14)

## File Paths (Android)

- Database: `/data/data/com.red5flag.red/files/sled/`
- Documents: `/data/data/com.red5flag.red/files/documents/`
- These are resolved via Tauri's `app_data_dir()` API
