# SpaceShift

[中文介绍](./README_CN.md)

SpaceShift is a powerful, lightweight Chrome Profile Management tool built with **Tauri**, **Rust**, and **Vue 3**. It's designed to help developer, marketers, and power users manage multiple Chrome profiles with ease, ensuring data isolation and high performance.

![SpaceShift Banner](https://raw.githubusercontent.com/chao-eng/SpaceShift/master/src-tauri/icons/128x128.png)

## ✨ Features

- **🚀 High Performance**: Built with Rust backend for near-instant startup and minimal resource usage.
- **🛡️ Data Isolation**: Each profile has its own dedicated data directory, keeping your browsing data completely separate.
- **📊 Performance Monitoring**: Real-time tracking of browser startup time, process creation, and DOM ready events.
- **💾 Easy Backup**: Built-in backup and restore functionality for your profile data.
- **🎨 Modern UI**: Beautiful "Liquid Glass" aesthetic inspired by modern productivity tools.
- **🌍 Internationalization**: Supports both English and Chinese.

## 🛠️ Tech Stack

- **Frontend**: Vue 3, Vite, TypeScript, Element Plus, Pinia, vue-i18n.
- **Backend**: Rust, Tauri, SQLite (via `rusqlite`).
- **Styling**: SCSS, Flexbox/Grid for responsive design.

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v16+)
- [Rust](https://www.rust-lang.org/) (latest stable)
- [Tauri Dependencies](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/chao-eng/SpaceShift.git
   cd SpaceShift
   ```

2. Install dependencies:
   ```bash
   yarn install
   ```

3. Run in development mode:
   ```bash
   yarn tauri dev
   ```

4. Build for production:
   ```bash
   yarn tauri build
   ```

## � Usage

1. **Create Profile**: Click the "New Profile" button and specify a name and optional icon/tags.
2. **Launch Chrome**: Click the play icon on a profile card to start a dedicated Chrome instance.
3. **Monitor Performance**: After launching, use the "Performance" action to see detailed startup metrics.
4. **Backup Data**: Periodically backup your profiles to ensure no data loss.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## � License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Created with ❤️ by [chao-eng](https://github.com/chao-eng)
