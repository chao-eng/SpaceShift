# SpaceShift - Chrome 多配置管理专家

[English Version](./README.md)

[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen?logo=vue.js)](https://vuejs.org/)
[![Performance](https://img.shields.io/badge/Optimization-Enabled-orange)](https://github.com/guojc/SpaceShift)

**SpaceShift** 是一款基于 **Tauri** + **Vue 3** 开发的现代化 Chrome 配置管理器。它专注于为开发者、测试人员以及需要多账号隔离的用户提供简洁、高效、且具备性能优化能力的 Chrome 浏览器环境管理方案。

---

## ✨ 核心特性

- 🚀 **一键快速启动**：深度集成系统底层，秒级唤起指定 Chrome 实例。
- 🛡️ **账号完全隔离**：利用 Chrome 原生 Profile 隔离机制，确保每个配置的缓存、Cookies、历史记录和插件完全独立。
- 📊 **性能监控分析**：内置性能采集引擎，自动分析 Chrome 启动耗时、DNS 查询、TCP 连接等关键指标。
- 🛠️ **启动参数优化**：自动配置内网加速、禁用冗余服务等优化参数，显著降低内存占用与启动延迟。
- 📦 **配置备份同步**：支持一键打包备份 Profile 数据，轻松实现数据迁移与环境克隆。
- 🎨 **飞书风格 UI**：极致简约的界面设计，支持文件夹分类（通过标签）、搜索过滤、网格/列表视图切换。
- 📁 **目录快速直达**：在界面中直接打开物理存储路径，方便手动调整配置。

## 🛠️ 技术架构

- **前端 (Frontend)**: [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Element Plus](https://element-plus.org/)
- **后端 (Backend)**: [Rust](https://www.rust-lang.org/) + [Tauri](https://tauri.app/)
- **数据库 (Database)**: SQLite (用于高效存储配置元数据)
- **构建工具 (Bundle Tool)**: [Vite](https://vitejs.dev/)

## 🚀 快速开始

### 运行环境
- Node.js (v18+)
- Rust (Stable 1.75+)
- Google Chrome (需安装在系统默认路径)

### 开发调试
```bash
# 克隆项目
git clone https://github.com/guojc/SpaceShift.git
cd SpaceShift

# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 生产发布
```bash
# 构建正式包 (Windows/macOS/Linux)
npm run tauri build
```

## 📸 界面预览

*SpaceShift 提供了清新、直观的操作界面，让复杂的配置管理变得像使用文件管理器一样自然。*

> **提示**：如需自定义启动页或注入额外参数，可在“编辑配置”窗口中快速设置。

## 📝 性能报告示例
当您通过 SpaceShift 启动 Chrome 时，系统会监控以下性能指标：
- **Total Launch Duration**: 总启动耗时
- **Chrome Process Spawn**: 进程创建耗时
- **DNS Lookup / TCP Connect**: 网络预热耗时

这些数据将帮助您发现影响浏览器速度的潜在因素，并提供优化建议。

## 💡 常见问题 (FAQ)

**Q: 我的配置数据存储在哪里？**
A: 默认存储在系统的 `AppData` (Windows) 或 `Application Support` (macOS) 目录下的 `SpaceShift/profiles` 文件夹中。您可以通过界面的“打开目录”按钮直接访问。

**Q: 是否支持除 Chrome 之外的其他浏览器（如 Edge, Brave）？**
A: 目前版本深度优于 Chrome。虽然底层逻辑相似，但暂未正式适配其他内核浏览器，建议优先配套 Google Chrome 使用。

**Q: 如何自定义启动页面？**
A: 您可以在“编辑配置”中设置默认启动 URL。此外，也可以配合启动参数注入特定的 flags（如 `--incognito` 无痕模式）。

**Q: 备份文件是否包含插件？**
A: 是的。一键备份功能会完整打包整个 Profile 目录，包括已安装的扩展程序、书签、历史记录及登录状态。

## 🛡️ 安全与隐私

SpaceShift 仅作为本地浏览器进程的管理中介，**不会**上传您的任何浏览器数据（如 Cookies、密码等）到云端。所有数据均存储在您的本地设备中。

## 📜 许可证

本项目基于 [MIT License](LICENSE) 许可。

---
*由 SpaceShift 团队驱动，让您的浏览器工作流更上一层楼。*
