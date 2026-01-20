# <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/rocket.svg" width="30" height="30" style="vertical-align: middle"/> Dev Stack Launcher

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=white)
![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)
![Version](https://img.shields.io/badge/version-2.0.0-blue?style=for-the-badge)

> **The ultimate developer companion for the Datakeen stack — V2.0.0**
> Stop juggling multiple terminal windows. Manage your full-stack environment from a single, beautiful native interface powered by Tauri + xterm.js.

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/image.svg" width="24" height="24" style="vertical-align: middle"/> Preview

![Dev Launcher Screenshot](./assets/preview2.gif)

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/download.svg" width="24" height="24" style="vertical-align: middle"/> Download

| Windows | macOS |
| :--- | :--- |
| [![Windows](https://img.shields.io/badge/Download-Windows-0078D4?style=for-the-badge&logo=windows11&logoColor=white)](https://github.com/P4ST4S/rust-developer-tool-web/releases/download/v2.0.0/Dev.Stack.Launcher_2.0.0_x64-setup.exe) | [![macOS](https://img.shields.io/badge/Download-macOS-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/P4ST4S/rust-developer-tool-web/releases/download/v2.0.0/Dev.Stack.Launcher_2.0.0_universal.dmg) |

> **Note**: Binaries are automatically built via GitHub Actions for each release.

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/sparkles.svg" width="24" height="24" style="vertical-align: middle"/> Features

### <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/gamepad-2.svg" width="20" height="20" style="vertical-align: middle"/> Unified Control Center

Start, stop, and restart your **Frontend** (Vite) and **Backend** services independently with a single click. No more `Ctrl+C` confusion.

### <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/terminal.svg" width="20" height="20" style="vertical-align: middle"/> Professional Terminal Experience

Powered by **xterm.js** (same engine as VS Code), providing a native terminal experience in a GUI.

- **Professional text handling**:
  - ✅ Native text selection with auto-scroll (drag beyond visible area)
  - ✅ Perfect ANSI color rendering (no manual parsing needed)
  - ✅ System-native copy/paste
  - ✅ Clickable URLs and file paths
- **Smart filtering**: Filter by source (Frontend/Backend/System) and level (Normal/Error)
- **Advanced search** (xterm.js native):
  - Incremental search with highlighting
  - Navigate matches with shortcuts
  - Case-sensitive/insensitive options
- **Performance**: Handle 100k+ lines smoothly with virtual scrolling
- **Word wrap**: Automatic line wrapping for long entries
- **Scrollback**: Configurable history (default: 10,000 lines)

### <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/brain-circuit.svg" width="20" height="20" style="vertical-align: middle"/> Smart Integration

- **Auto-Discovery**: Automatically detects running ports and Vite dev server URLs
- **One-Click Open**: Launch your browser directly to the correct local URL
- **Graceful Shutdown**: Handles process groups correctly (`SIGTERM`/`SIGKILL`) ensuring no zombie processes

### <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/palette.svg" width="20" height="20" style="vertical-align: middle"/> Modern UI

- Built with **Tauri 2** for native performance with web flexibility
- **Dark/Light Mode** support with xterm.js themes
- Clean, modern aesthetics
- Native window controls and system integration

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/layers.svg" width="24" height="24" style="vertical-align: middle"/> Architecture & Tech Stack

### V2.0.0 - Hybrid Architecture (Tauri)

```
┌─────────────────────────────────────┐
│         Frontend (Web)              │
│  TypeScript + xterm.js              │
│  - Terminal rendering               │
│  - UI Controls                      │
│  - State management                 │
└────────────┬────────────────────────┘
             │ IPC (Tauri Commands)
             │ Events (Log streaming)
┌────────────▼────────────────────────┐
│         Backend (Rust)              │
│  Tauri + Tokio                      │
│  - Process management               │
│  - Log capture & streaming          │
│  - System commands                  │
└─────────────────────────────────────┘
```

### Tech Stack

**Backend (Rust)**:
- **[Tauri 2](https://tauri.app/)** - Modern desktop framework
- **[Tokio](https://tokio.rs/)** - Async runtime for process management
- **[Serde](https://serde.rs/)** - Serialization for IPC

**Frontend (Web)**:
- **[TypeScript](https://www.typescriptlang.org/)** - Type-safe vanilla JS
- **[xterm.js](https://xtermjs.org/)** - Professional terminal emulator
  - `xterm-addon-fit` - Auto-resize
  - `xterm-addon-search` - Search functionality
  - `xterm-addon-web-links` - Clickable URLs

**Binary Size**: ~5MB (includes WebView runtime)
**Memory Usage**: ~80-120MB (WebView + xterm.js)

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/zap.svg" width="24" height="24" style="vertical-align: middle"/> What's New in V2.0.0

### 🔄 Complete Rewrite

- ✅ Migrated from **egui** to **Tauri + xterm.js**
- ✅ Replaced custom log rendering with professional terminal emulator
- ✅ Native text selection with auto-scroll (finally!)
- ✅ Better ANSI color support (terminal-native)
- ✅ Improved performance for large log volumes
- ✅ Professional terminal UX (same as VS Code)

### 🎯 Key Improvements

| Feature | V1 (egui) | V2 (Tauri) |
|---------|-----------|------------|
| Text Selection | ❌ No auto-scroll | ✅ Native auto-scroll |
| ANSI Colors | Manual parsing | ✅ Terminal native |
| Large logs (100k lines) | Laggy | ✅ Smooth (virtual scroll) |
| Copy/Paste | Manual | ✅ System native |
| Clickable URLs | No | ✅ Yes |
| Bundle size | 3MB | 5MB |
| Memory | 30MB | 100MB |

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/play.svg" width="24" height="24" style="vertical-align: middle"/> Getting Started

### Prerequisites

- **Rust & Cargo** (latest stable)
- **Node.js 18+** & **pnpm**
- **System dependencies**:
  - macOS: Xcode Command Line Tools
  - Linux: `webkit2gtk`, `libayatana-appindicator`
  - Windows: WebView2 (usually pre-installed)

### Development

```bash
# Navigate to the project
cd rust-gui

# Install frontend dependencies
pnpm install

# Run in development mode (hot reload enabled)
pnpm run dev

# Backend Rust code: src-tauri/src/
# Frontend TypeScript code: ui/
```

### Production Build

```bash
# Build optimized release
pnpm run build

# Output:
# - macOS: src-tauri/target/release/bundle/macos/
# - Linux: src-tauri/target/release/bundle/appimage/
# - Windows: src-tauri/target/release/bundle/msi/
```

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/folder.svg" width="24" height="24" style="vertical-align: middle"/> Project Structure

```
rust-gui/
├── ui/                    # Frontend (TypeScript)
│   ├── index.html        # Entry point
│   ├── main.ts           # Application logic
│   ├── terminal.ts       # xterm.js integration
│   └── styles.css        # Styling
├── src-tauri/            # Backend (Rust)
│   ├── src/
│   │   ├── main.rs      # Tauri app entry
│   │   ├── commands.rs  # IPC commands
│   │   ├── process.rs   # Process management
│   │   └── logs.rs      # Log utilities
│   ├── Cargo.toml
│   └── tauri.conf.json  # Tauri configuration
├── package.json
└── MIGRATION.md         # Migration guide from V1
```

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/wrench.svg" width="24" height="24" style="vertical-align: middle"/> Troubleshooting

### Port Already in Use

The tool attempts to gracefully kill process groups. If a process persists:

```bash
# Find the process
lsof -i :3000  # or :5173

# Kill it
kill -9 <PID>
```

### Frontend Won't Start

Ensure you're in the correct directory:

```bash
cd /path/to/datakeen-refacto/rust-gui
pnpm install
pnpm run dev
```

### Build Errors

Make sure all system dependencies are installed:

```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.0-devel \
  openssl-devel \
  curl \
  wget \
  libappindicator-gtk3-devel \
  librsvg2-devel
```

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/map.svg" width="24" height="24" style="vertical-align: middle"/> Roadmap

- [x] **V2.0: Tauri Migration** ✅
  - [x] Native text selection with auto-scroll
  - [x] xterm.js integration
  - [x] Professional terminal UX
- [ ] **Git Operations Tab**: Dedicated tab for git workflows
- [ ] **Branch Management**: Create, checkout, and switch branches
- [ ] **Quick Pull**: One-click pull for main branches
- [ ] **Commit Interface**: Stage and commit with message input
- [ ] **Push Button**: Sync local changes to remote

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/book-open.svg" width="24" height="24" style="vertical-align: middle"/> Documentation

- [Migration Guide](./MIGRATION.md) - How we migrated from egui to Tauri
- [Tauri Documentation](https://tauri.app/v2/guides/)
- [xterm.js Documentation](https://xtermjs.org/)

---

## <img src="https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/star.svg" width="24" height="24" style="vertical-align: middle"/> Why V2?

**Problem with V1 (egui)**:
- Text selection without auto-scroll was frustrating
- Manual ANSI parsing was complex and incomplete
- Limited to egui's text rendering capabilities

**Solution with V2 (Tauri + xterm.js)**:
- ✅ Professional terminal experience (same as VS Code)
- ✅ Native text selection with auto-scroll
- ✅ Perfect ANSI color rendering
- ✅ Better performance for large logs
- ✅ Modern development workflow

**Trade-offs accepted**:
- Slightly larger binary (5MB vs 3MB) - worth it
- Higher memory usage (100MB vs 30MB) - modern standards
- Web stack dependency - but simpler maintenance

---

**Built with ❤️ for developers who hate managing terminals**
