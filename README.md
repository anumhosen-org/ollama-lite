<p align="center">
  <img src="src/assets/icon.png" alt="Ollama Lite Logo" width="100" height="100" style="border-radius: 20px;" />
</p>

<h1 align="center">Ollama Lite</h1>

<p align="center">
  <strong>High-Performance Local AI Desktop Runner with 100% Ollama API & CLI Drop-In Compatibility</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-v2-blue?logo=tauri&logoColor=white" alt="Tauri v2" />
  <img src="https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=black" alt="React 19" />
  <img src="https://img.shields.io/badge/llama.cpp-Inference%20Engine-ff9900?logo=cplusplus&logoColor=white" alt="llama.cpp" />
  <img src="https://img.shields.io/badge/Ollama%20API-Port%2011434-green?logo=ollama&logoColor=white" alt="Ollama API 11434" />
  <img src="https://img.shields.io/badge/TailwindCSS-v4-38bdf8?logo=tailwindcss&logoColor=white" alt="TailwindCSS v4" />
  <img src="https://img.shields.io/badge/Platforms-Windows%20%7C%20macOS%20%7C%20Linux-blueviolet" alt="Platforms" />
  <img src="https://img.shields.io/badge/License-MIT-purple" alt="License" />
</p>

---

## 🌟 Overview

**Ollama Lite** is a lightweight, self-contained local AI desktop runner built with **Tauri v2** and **React 19** for **Windows, macOS, and Linux**. It marries the developer-centric UI/UX and REST API semantics of **Ollama** with the raw execution speed, GPU flexibility (Metal on macOS, Vulkan on Windows/Linux, CUDA on Windows, CPU fallback), and direct GGUF model control of **llama.cpp**.

Ollama Lite operates simultaneously as:
1. A **modern desktop application** with full chat workspace, collapsible reasoning view for DeepSeek-R1, and 23 curated models.
2. An **embedded REST reverse proxy on port `11434`**, providing 100% drop-in API parity with official Ollama.
3. A **drop-in terminal CLI (`ollama`)** installed directly into your system PATH (PowerShell & Batch on Windows; executable shell script on macOS & Linux).

---

## 🚀 Key Features

### 🔌 100% Ollama REST API Compatibility (`:11434`)
Ollama Lite runs a high-performance, asynchronous HTTP proxy on `http://127.0.0.1:11434`. Any tool, extension, or library built for Ollama or OpenAI works out of the box with zero configuration:
- **Cursor IDE** & **Continue.dev**
- **LangChain**, **LlamaIndex**, & **LiteLLM**
- **Open WebUI** & third-party AI web interfaces
- **Official Ollama Python & JavaScript SDKs**

| Endpoint | Method | Description |
| :--- | :---: | :--- |
| `/` | `GET`, `HEAD` | Server heartbeat handshake (`"Ollama is running\n"`) |
| `/api/version` | `GET`, `HEAD` | Daemon version response (`{"version": "0.1.0"}`) |
| `/api/tags` | `GET`, `HEAD` | Catalog of local GGUF models in Ollama JSON format |
| `/api/ps` | `GET` | Active model process telemetry, VRAM usage, and status |
| `/api/show` | `POST` | Model architecture, parameters, template, system, license |
| `/api/chat` | `POST` | Multi-turn chat completion with streaming NDJSON chunks |
| `/api/generate` | `POST` | Raw prompt completions with streaming NDJSON tokens |
| `/api/pull` | `POST` | Download models with streaming NDJSON progress chunks |
| `/api/delete` | `DELETE` | Unload and delete model weights from disk |
| `/api/copy` | `POST` | Copy or alias local model references |
| `/api/embed` | `POST` | Generate text vector embeddings |
| `/api/embeddings` | `POST` | Legacy vector embeddings endpoint |
| `/v1/models` | `GET` | OpenAI-compatible model catalog |
| `/v1/*` | `POST` | Transparent OpenAI proxy (`/v1/chat/completions`, etc.) |

---

### 💻 Drop-In Terminal CLI (`ollama`)
Install the terminal CLI in 1-click from the Settings tab. It installs both `ollama.cmd` (primary drop-in replacement) and `ollama-lite.cmd` (alias) to your user `PATH`:

```powershell
# Check installed version
ollama version

# List installed models
ollama list
ollama ls

# Pull models directly from curated library or Hugging Face
ollama pull deepseek-r1:1.5b
ollama pull llama3.2:3b

# Run an interactive streaming chat REPL
ollama run deepseek-r1:1.5b

# Run single-shot prompt execution
ollama run llama3.2 "Explain quantum computing in three sentences"

# Inspect model Modelfile, template, and parameters
ollama show deepseek-r1:1.5b
ollama show llama3.2 --modelfile
ollama show llama3.2 --parameters

# Check active GPU process and memory
ollama ps

# Copy or alias a model
ollama cp llama3.2:3b my-assistant:latest

# Remove an installed model
ollama rm my-assistant:latest

# Unload active model from memory
ollama stop

# Verify server status
ollama serve
```

---

### 🧠 Embedded Inference Engine & Hardware Auto-Tuning
- **Supervised `llama-server` Sidecar**: Manages the inference lifecycle on private loopback port `11435`.
- **Automatic Hardware Profiling**: Detects CPU brand and cores, system RAM, discrete GPU (NVIDIA, AMD, Intel), and estimated VRAM.
- **Hardware-Tailored Recommendations**: Suggests optimal parameter size, quantization levels, and context limits based on your hardware specs.
- **Integrated Binary Downloader**: Automatically tracks, downloads, and unpacks official `llama.cpp` GitHub releases (Vulkan, CUDA, CPU) into `%APPDATA%/ollama-lite/bin/`.

---

### 📦 Curated Model Hub (23 Models)
One-click model downloading with real-time transfer speed, progress bars, and Hugging Face GGUF imports:
- **DeepSeek**: DeepSeek R1 (1.5B, 7B, 8B Llama), DeepSeek Coder V2 Lite (16B MoE)
- **Meta Llama**: Llama 3.2 (1B, 3B), Llama 3.1 (8B), Llama 2 (7B Chat)
- **Qwen**: Qwen 2.5 Coder (1.5B, 7B, 14B), Qwen 2.5 (0.5B, 3B, 7B)
- **Mistral**: Mistral 7B Instruct v0.3, Mistral Nemo 12B
- **Microsoft Phi**: Phi-4 (14B), Phi-3.5 Mini (3.8B), Phi-3 Mini
- **Google Gemma**: Gemma 2 (2B, 9B)
- **Hermes & SmolLM**: Hermes 3 Llama 3.1 (8B), SmolLM2 (1.7B, 360M)
- **Embeddings**: Nomic Embed Text v1.5

---

### 🎨 Modern Desktop Interface
- **Dark / Light Theme**: Clean Tailwind CSS gray-scale palette with system synchronization and flash-free early hydration.
- **Reasoning Thinking Accordion**: Expandable/collapsible thought process visualization (`<think>...</think>`) designed for DeepSeek-R1 and chain-of-thought models.
- **Markdown & Code Highlighting**: Syntax-highlighted code blocks with 1-click clipboard copy.
- **Hyperparameter Drawer**: Granular sliders for temperature, context length (`num_ctx`), top-p, top-k, GPU offload layers (`-ngl`), and custom system prompts.
- **System Tray Minimization**: Closing the window (via **✕** or **Alt+F4**) minimizes Ollama Lite to the Windows system tray, keeping inference and the REST API active in the background. Left-clicking the tray icon restores the window.

---

## 🛠️ Tech Stack & Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      OLLAMA LITE                        │
├────────────────────────────┬────────────────────────────┤
│       REACT 19 UI          │     RUST BACKEND (Tauri)   │
│ • Tailwind CSS v4          │ • Tokio Async REST Server  │
│ • React Icons (VscChrome)  │ • Model Downloader & GGUF  │
│ • Markdown & Syntax High.  │ • SQLite DB (Conversations)│
│ • Settings & Telemetry     │ • Win Registry & Autostart │
└─────────────┬──────────────┴─────────────┬──────────────┘
              │ IPC (Tauri v2 invoke)       │
              ▼                             ▼
┌────────────────────────────┐┌───────────────────────────┐
│    REST API (Port 11434)   ││   llama-server Sidecar    │
│ • Full Ollama API Proxy    ││ • Vulkan / CUDA / CPU     │
│ • OpenAI /v1/ Proxy        ││ • Loopback Port 11435     │
│ • Drop-in 'ollama' CLI     ││ • Dynamic Model Booting   │
└────────────────────────────┘└───────────────────────────┘
```

- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS v4, `react-icons`
- **Desktop Runtime**: Tauri v2, WebView2 (Windows)
- **Backend Services**: Rust (Tokio, Reqwest, Serde, Rusqlite, Sysinfo)
- **Inference**: Official `llama.cpp` (`llama-server`) with Metal, Vulkan, and CUDA acceleration

---

## 📥 Installation

### Download Pre-Built Binaries
Download the latest native binary package for your operating system from the [Releases](https://github.com/your-username/ollama-lite/releases) page:

- **Windows**:
  - `Ollama-Lite_<version>_x64-setup.exe` (Recommended NSIS Installer)
  - `Ollama-Lite_<version>_x64_en-US.msi` (MSI Package)
- **macOS**:
  - `Ollama-Lite_<version>_aarch64.dmg` (Apple Silicon M1/M2/M3/M4 with Metal acceleration)
  - `Ollama-Lite_<version>_x64.dmg` (Intel Mac)
- **Linux**:
  - `Ollama-Lite_<version>_amd64.AppImage` (Universal standalone Linux package)
  - `Ollama-Lite_<version>_amd64.deb` (Debian / Ubuntu package)

---

## 💻 Building from Source

### Prerequisites
1. **Node.js**: `v20.x` or later ([Download](https://nodejs.org/))
2. **Rust**: Stable toolchain ([rustup.rs](https://rustup.rs/))
3. **Platform Dependencies**:
   - **Windows**: Visual Studio Build Tools with C++ workload
   - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
   - **Linux (Ubuntu / Debian)**:
     ```bash
     sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf build-essential
     ```

### Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-username/ollama-lite.git
   cd ollama-lite
   ```

2. **Install frontend dependencies**:
   ```bash
   npm install
   ```

3. **Run in development mode**:
   ```bash
   npm run tauri dev
   ```

4. **Build production bundle**:
   ```bash
   npm run tauri build
   ```
   Installers will be generated in `src-tauri/target/release/bundle/`.

---

## 🤖 Automated Releases via GitHub Actions

This repository includes a multi-platform GitHub Actions workflow ([`.github/workflows/release.yml`](.github/workflows/release.yml)) for automated builds and releases across Windows, macOS, and Ubuntu Linux.

### Creating a Release:
1. **Push a Git Tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
2. **Or Manual Trigger**:
   - Navigate to the **Actions** tab on GitHub.
   - Select the **Release** workflow.
   - Click **Run workflow**, optionally entering a version tag (e.g., `v0.1.0`).

The workflow will automatically compile the frontend, build the optimized Rust release bundles in parallel across Windows, macOS, and Linux runners, and upload all `.exe`, `.msi`, `.dmg`, `.AppImage`, and `.deb` packages to a unified GitHub Release.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
Inference engine powered by [llama.cpp](https://github.com/ggerganov/llama.cpp). UI inspired by [Ollama](https://ollama.com).
