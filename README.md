# SpaceShift

SpaceShift is a modern, high-performance, cross-platform multi-profile browser management tool built with **Rust**, **Tauri v2**, and **Vue 3**. It allows you to seamlessly create, manage, and launch multiple isolated Google Chrome profiles on a single machine.

[中文文档 (Chinese)](README_CN.md)

## ✨ Features

- 🚀 **Hardware-Accelerated Core**: Built on Tauri v2 and Rust, offering an extremely low memory footprint compared to Electron-based alternatives.
- 📦 **Total Data Isolation**: Each profile receives its own dedicated `User Data Directory`, ensuring complete separation of cookies, local storage, extensions, and cache.
- 🌗 **Native Dark Mode Sync**: Flawless dark mode that instantly synchronizes the internal UI with the native OS window borders (Windows & macOS).
- 📊 **Real-time Performance Metrics**: Connects dynamically via Chrome DevTools Protocol (CDP) to measure exact page load, DOM ready, and TCP/DNS timings for your profiles.
- 🛠 **System-Aware Optimizations**: Automatically applies platform-specific memory and network optimization flags (e.g., PowerShell memory checks on Windows, `/proc/meminfo` on Linux) to prevent system hangs.
- 🗄 **Backup & Restore**: Easily snapshot and restore your profile directories with a single click.
- 🌐 **Internationalization**: Fully localized in English and Simplified Chinese out of the box.
- ⚡ **Batch Operations**: Multi-select support to staggered-launch multiple profiles at once without spiking your CPU.
- 🛡 **Smart Lock Release**: Built-in repair utility to clean up orphaned Chrome lockfiles (`SingletonLock`, `Parent.lock`) when browser processes crash abruptly.

## 📦 Tech Stack

- **Frontend**: Vue 3 (Composition API), Vite, TypeScript, Element Plus, Pinia, Vue-i18n, SCSS.
- **Backend / Core**: Rust, Tauri v2.
- **System Integration**: `winreg` (Windows path resolving), custom CDP WebSockets (`tokio-tungstenite`), `sysinfo`.

## 🚀 Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/) (v16+)
- [Rust](https://rustup.rs/) (latest stable)
- **Chrome/Chromium** installed on your system. SpaceShift will automatically scan common installation paths and registry entries to find it.

### Installation & Development

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/spaceshift.git
   cd spaceshift
   ```

2. Install frontend dependencies:
   ```bash
   yarn install
   # or npm install / pnpm install
   ```

3. Start the development server (runs Vite + Tauri backend):
   ```bash
   yarn tauri dev
   ```

### Building for Production

Compile a native executable for your current OS:
```bash
yarn build
```
*(The compiled binaries will be generated in `src-tauri/target/release/bundle/`)*

## 💡 Usage Highlights

- **Creating a Profile**: Click "New Profile" to create an isolated environment. You can specify a custom Chrome path or let SpaceShift auto-detect it.
- **Monitoring**: After launching a profile, click the "..." menu and select "Performance" to view real-time CDP timings for recent launches.
- **Repair**: If a profile is stuck and won't launch, click "Repair (Unlock)" to wipe lingering Chromium lockfiles.
- **Batch Launch**: Check multiple profile cards to trigger the batch operations toolbar, allowing you to launch several instances with built-in startup delays.

## 📝 License

This project is open-sourced under the MIT License.
Created by bujic.
