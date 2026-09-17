# BranchAI 🌿

**BranchAI** is a modern, lightweight, high-performance desktop Git client built with **Tauri v2**, **Vue 3**, **TypeScript**, and **Rust**. It combines a clean Atomic Design interface with AI assistance to simplify branch management, commit generation, code review, and multi-account Git authentication.

---

## 🚀 Key Features

- 📜 **Interactive Commit & Branch Graph**: Visualize commit history, branch splits, merge nodes, and HEAD pointers with high clarity.
- 🔑 **Multi-Account Git Management**: Easily switch between personal, work, or custom Git credentials (SSH Keys & Personal Access Tokens) on a per-repository basis.
- 🤖 **AI-Assisted Workflows**:
  - **Smart Commit Generation**: Generate concise, contextual commit messages from staged diffs using AI models.
  - **AI Conflict Resolver**: Get intelligent recommendations and automated resolution paths for complex Git merge conflicts.
  - **Code Review Assistant**: Perform automated static code reviews on changes before opening pull requests.
- 🔀 **Branch & Stash Management**: Create, switch, merge, and discard changes safely. Full support for Git stashing and unstashing.
- 🔍 **Diff & Blame Viewer**: Inspect staged/unstaged changes, inline diff patches, file history, and line-by-line `git blame`.
- 📋 **Integrated Pull Requests & Issues**: Manage repository pull requests and issues locally within the app UI.
- 🎨 **Atomic Component System**: Modular, responsive UI architecture designed with custom dark-mode aesthetics and fluid micro-animations.

---

## 🛠️ Technical Stack

### **Frontend**
- **Framework**: [Vue 3](https://vuejs.org/) (`<script setup>` Composition API)
- **Language**: [TypeScript](https://www.typescriptlang.org/)
- **Build Tool**: [Vite](https://vitejs.dev/)
- **Architecture**: Atomic Component Design (`atoms`, `molecules`, `organisms`)
- **Icons**: [Lucide Vue Next](https://lucide.dev/guide/packages/lucide-vue-next)
- **Styling**: Vanilla CSS Design System (Custom variables, glassmorphism, responsive grid layouts)

### **Backend (Desktop Native)**
- **Framework**: [Tauri v2](https://v2.tauri.app/)
- **Language**: [Rust](https://www.rust-lang.org/)
- **Git Engine**: [`git2`](https://crates.io/crates/git2) (Rust bindings to `libgit2`)
- **Async Runtime**: [`tokio`](https://tokio.rs/)
- **Networking**: [`reqwest`](https://crates.io/crates/reqwest) (JSON API integrations for AI services)
- **Native Dialogs**: [`rfd`](https://crates.io/crates/rfd) (Native file & folder pickers)

---

## 📋 Prerequisites

Before building BranchAI, ensure you have the following installed on your machine:

1. **Node.js** (v18.0 or higher) & `npm`
2. **Rust & Cargo** (v1.75 or higher) — Install via [rustup.rs](https://rustup.rs/)
3. **Platform Build Dependencies** (Required for Tauri v2):
   - **Linux (Ubuntu/Debian)**:
     ```bash
     sudo apt update
     sudo apt install -y build-essential curl wget libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
     ```
   - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
   - **Windows**: C++ Build Tools for Visual Studio

---

## 📦 Installation & Setup

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/your-username/BranchAI.git
   cd BranchAI
   ```

2. **Install Frontend Dependencies**:
   ```bash
   npm install
   ```

---

## 💻 Development

To launch the desktop application in live-reloading development mode:

```bash
npm run tauri dev
```

If you only want to run the web frontend in browser mode:

```bash
npm run dev
```

---

## 🏗️ Production Build

### **Compile App Distribution Packages** (Native Desktop Binary)

To build the optimized production desktop executable (AppImage / deb / dmg / msi):

```bash
npm run tauri build
```

The output binaries will be generated inside:
`src-tauri/target/release/bundle/`

### **Frontend Type-Check & Verification**

To verify TypeScript types and build the web bundle:

```bash
npm run build
```

---

## 📁 Architecture Overview

```
BranchAI/
├── src/                          # Vue 3 Frontend Root
│   ├── assets/                   # CSS Design System & Static Assets
│   ├── components/               # Atomic Component Hierarchy
│   │   ├── atoms/                # Base UI Elements (Buttons, Inputs, Badges)
│   │   ├── molecules/            # Compound Controls (SearchBar, Modal, Tabs)
│   │   └── organisms/            # Feature Views (BranchGraph, DiffViewer, Settings)
│   ├── App.vue                   # Application Entry Shell
│   └── main.ts                   # Vue Application Bootstrap
│
├── src-tauri/                    # Rust Desktop Backend Root
│   ├── src/
│   │   ├── models/               # Domain DTO Data Models (Account, Commit, File, Branch)
│   │   ├── services/
│   │   │   ├── git/              # Modular Domain Git Services (diff, commit, branch, stash, remote, inspection)
│   │   │   ├── ai_service.rs     # AI Provider Integration Service
│   │   │   └── config_service.rs # Local Application Configuration Service
│   │   ├── lib.rs                # Tauri Command Registry & App Setup
│   │   └── main.rs               # Binary Execution Entrypoint
│   └── Cargo.toml                # Rust Dependencies & Tauri Manifest
```

---

## 📄 License

This project is licensed under the MIT License.