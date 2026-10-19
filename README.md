# 🐳 Easy Docker

<p align="center">
  <img src="https://raw.githubusercontent.com/Brunopaun/easy-docker/main/assets/logo.jpg" alt="Easy Docker Whale Mascot" width="400"/>
</p>

> A fast, lightweight, and modern Terminal User Interface (TUI) for managing Docker containers, images, volumes, and networks built in Rust.

[![Crates.io](https://img.shields.io/crates/v/easy-docker.svg)](https://crates.io/crates/easy-docker)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org)
[![UI: Ratatui](https://img.shields.io/badge/UI-Ratatui_v0.30-cyan.svg)](https://ratatui.rs)

---

## ⚡ Installation

### Homebrew (macOS & Linux)

```bash
brew install Brunopaun/easy-docker/easy-docker
```

---

### Shell Script (macOS & Linux, No Rust Required)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Brunopaun/easy-docker/releases/latest/download/easy-docker-installer.sh | sh
```

---

### Windows (PowerShell or Chocolatey)

**PowerShell:**
```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/Brunopaun/easy-docker/releases/latest/download/easy-docker-installer.ps1 | iex"
```

**Chocolatey:**
```powershell
choco install easy-docker
```

---

### Cargo / Cargo Binstall

Requires [Rust & Cargo](https://www.rust-lang.org/tools/install/):

```bash
# Instant binary install (no compilation)
cargo binstall easy-docker

# Or compile from source via crates.io
cargo install easy-docker
```

---

### Build from Source

Building from source requires Rust. If you don't have Rust installed yet, refer to the [Rust Installation Guide](https://www.rust-lang.org/tools/install/) or install via [rustup](https://rustup.rs/):

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
> **Windows:** Download and run [`rustup-init.exe`](https://rustup.rs/) or check the [installation instructions](https://www.rust-lang.org/tools/install/).

Once Rust is installed, clone and install:

```bash
git clone https://github.com/brunopassosaun/easy-docker.git
cd easy-docker
cargo install --path .
```

---

## ✨ Features

- 📁 **Docker Compose Auto-Grouping**: Automatically detects and groups containers by Docker Compose project labels. Collapse and expand groups seamlessly.
- ⚡ **Asynchronous & Responsive**: Built on `tokio` and `bollard` (Docker API client) for instant, non-blocking UI interactions.
- 🖱️ **Mouse & Keyboard Native**: Full mouse support (click to select/toggle) alongside vim-style (`j`/`k`) and arrow key navigation.
- 🛠️ **Full Resource Lifecycle Management**:
  - **Containers**: Start (`s`), Stop (`x`), Restart (`r`), Delete (`Shift+d`), Logs & Details (`l`/`d`).
  - **Images**: Inspect and Delete (`Shift+d`).
  - **Volumes**: Inspect and Delete (`Shift+d`).
  - **Networks**: Inspect and Delete (`Shift+d`).
- 🔍 **Container Inspector & Real-time Logs**: View live container logs or detailed metadata.
- 🗂️ **Tabbed Views**: Quick tab navigation (`1-4` or `Tab`) for Containers, Images, Volumes, and Networks.

---

## ⌨️ Controls & Shortcuts

| Key / Input | Action |
| :--- | :--- |
| **`↑` / `↓`** or **`k` / `j`** | Navigate through list / table |
| **Left Click** | Select item or toggle Compose group |
| **`Space`** / **`Enter`** | Expand or collapse Docker Compose project group |
| **`1` - `4`** / **`Tab`** | Switch active tabs (*Containers, Images, Volumes, Networks*) |
| **`l`** / **`d`** | Toggle between **Logs** and **Details** inspector view |
| **`s`** | **Start** selected container |
| **`x`** | **Stop** selected container |
| **`r`** | **Restart** selected container |
| **`Shift + d`** | **Delete** selected resource or group |
| **`q`** | **Quit** application |

---

## 🚀 Prerequisites

- **Docker Engine** running locally with standard Unix socket (`/var/run/docker.sock`) or default Windows pipe.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
