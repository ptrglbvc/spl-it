# spl-it

A lightweight desktop app for running Kotlin scripts (.kts) with real-time output streaming. Built with Tauri, Rust, React, and TypeScript.

## Features

- Simple code editor with Kotlin syntax highlighting
- Real-time console output streaming
- Gruvbox-inspired dark theme

## Installation

### Pre-built Releases

Download the latest release from the [GitHub Releases](https://github.com/ptrglbvc/spl-it/releases) page.

> **Note:** Currently only macOS builds are available.

### Build from Source

#### Prerequisites

- **Rust** 1.90 or higher
- **Node.js** or **Bun**
- **Kotlin** compiler (`kotlinc`) installed and available in PATH

#### Steps

```bash
# Clone the repository
git clone https://github.com/ptrglbvc/spl-it.git
cd spl-it

# Install dependencies
bun install
# or
npm install

# Run in development mode
bun tauri dev
# or
npm run tauri dev

# Build for production
bun tauri build
# or
npm run tauri build
```

## Usage

1. Write or paste Kotlin script code in the editor
2. Click **Run** to execute
3. View output in the console panel

## Requirements

The app requires `kotlinc` to be installed on your system:

- **macOS:** `brew install kotlin`
- **Linux:** `sudo snap install kotlin --classic`
- **SDKMAN:** `sdk install kotlin`

## License

MIT
