# Abel Music Player

一款轻量级本地音乐播放器.

## 下载

[下载链接](https://github.com/dzheming/abel-music-player/releases)

## 特性

- **本地播放** — 支持播放本地 MP3 FLAC WAV OGG AAC M4A WMA等文件
- **乐库管理** — 设置本地乐库,刷新乐库,分类展示
- **播放列表** — 支持创建管理播放列表
- **歌词同步** — 关联本地歌词文件,网上自动下载歌词
- **音效设置** — 支持调节EQ音效
- **便携模式** — 单EXE文件运行,无需安装

## 技术

- **前端** — Vue3 + TypeScript + Pinia + Vue Router
- **后端** — Rust + Tauri 2
- **数据** — SQLite (rusqlite) + 元数据缓存 + 播放列表 + 用户设置
- **解码** — lofty (ID3 标签)
- **样式** — 全部自定义CSS

## 构建

需要安装 [Node.js](https://nodejs.org/) 和 [Rust](https://www.rust-lang.org/)

```bash
npm install
cd src-tauri
npx tauri build
```
输出: `src-tauri/target/release/abelmp.exe`

## 开发

```bash
cd src-tauri
npx tauri dev
```
