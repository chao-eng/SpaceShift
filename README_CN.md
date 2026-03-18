# SpaceShift

SpaceShift 是一款现代、高性能、跨平台的多配置浏览器管理工具，基于 **Rust**、**Tauri v2** 和 **Vue 3** 构建。它能让你在同一台设备上无缝创建、管理并启动多个完全隔离的 Google Chrome 浏览器配置环境。

[English Documentation (英文文档)](README.md)

## ✨ 特性

- 🚀 **硬件加速内核**: 基于 Tauri v2 和 Rust 构建，相较于基于 Electron 的替代方案，内存占用极低。
- 📦 **完全的数据隔离**: 每个配置文件都有其独立的`用户数据目录`(User Data Directory)，确保 Cookie、本地存储、扩展和缓存完全物理隔离。
- 🌗 **原生暗色模式同步**: 完美的暗色模式，将应用内部的 UI 主题与原生操作系统窗口边框（Windows 和 macOS）瞬间同步切换。
- 📊 **实时性能监控**: 通过 Chrome 开发者工具协议（CDP）动态连接，准确测量所有配置文件的页面加载、DOM Ready 以及 TCP/DNS 等网络底层耗时。
- 🛠 **系统感知优化**: 自动应用特定于平台的内存和网络优化标志（例如 Windows 上的 PowerShell 内存检查、Linux 上的 `/proc/meminfo`）防止由于内存不足导致的系统卡顿。
- 🗄 **备份与恢复**: 一键轻松快照并恢复您的浏览器配置文件目录。
- 🌐 **国际化支持**: 开箱即用的英文和简体中文支持。
- ⚡ **批量操作**: 支持多选并同时启动多个配置文件，内置交错启动延迟算法以防止 CPU 数据飙升。
- 📡 **CDP 转发**: 可配置专门的“转发端口”（监听 0.0.0.0），将远端流量代理至浏览器本地。该功能已针对 WebSocket 进行深度优化（模拟 `socat fork,reuseaddr` 机制、`TCP_NODELAY` 以及 `TCP-KeepAlive` 探针），确保在复杂的网络环境下（如 NAT 驱逐）连接依然如磐石般稳固，即便长时间空闲也不会断连。
- 🛡 **智能解锁修复**: 内置修复实用程序，在浏览器进程异常崩溃时清理遗留的 Chrome 锁定文件（`SingletonLock`，`Parent.lock`）。

## 📦 技术栈

- **前端**: Vue 3 (Composition API)、Vite、TypeScript、Element Plus、Pinia、Vue-i18n、SCSS。
- **后端 / 核心**: Rust、Tauri v2。
- **系统集成**: `winreg` (Windows 路径解析)、自定义 CDP WebSockets (`tokio-tungstenite`)、`sysinfo`。

## 🚀 快速开始

### 环境依赖
- [Node.js](https://nodejs.org/) (v16+)
- [Rust](https://rustup.rs/) (最新稳定版)
- 系统中已安装 **Chrome/Chromium**。 SpaceShift 会自动扫描常见安装路径和注册表项来查找它。

### 安装与开发

1. 克隆仓库:
   ```bash
   git clone https://github.com/yourusername/spaceshift.git
   cd spaceshift
   ```

2. 安装前端依赖:
   ```bash
   yarn install
   # 或者使用 npm install / pnpm install
   ```

3. 启动开发服务器 (运行 Vite + Tauri 后端):
   ```bash
   yarn tauri dev
   ```

### 生产环境构建

编译对应你当前操作系统的原生可执行文件:
```bash
yarn tauri build
```
*(编译完成的二进制文件将在 `src-tauri/target/release/bundle/` 目录中生成)*

## 💡 使用亮点

- **创建新配置**: 点击“新建配置”创建一个隔离的环境。您可以指定自定义 Chrome 路径或让 SpaceShift 自动检测。
- **性能监控**: 启动配置后，点击“...”菜单并选择“运行性能”以查看近期启动进程的实时 CDP 耗时数据。
- **修复环境**: 如果配置文件卡住并且无法启动，请单击“修复配置 (解锁)”以擦除遗留的 Chromium 锁定文件。
- **批量启动**: 勾选多张配置卡片以触发批量操作工具栏，允许您一键同时启动多个实例。
- **远程自动化**: 在创建或编辑配置时设置 **CDP 转发端口**。这允许您通过局域网内的其他设备使用 Puppeteer 或 Playwright 访问该端口来远程控制浏览器实例。

## 📝 许可协议

本项目基于 MIT 许可证开源。
由 bujic 创作。
