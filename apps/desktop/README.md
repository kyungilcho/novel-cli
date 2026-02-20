# Tauri + React + TypeScript (Rsbuild)

This app uses Tauri + React + TypeScript with Rsbuild (powered by Rspack).

## Why Rsbuild (Rspack)

We chose Rsbuild (Rspack) over Vite for this desktop app.

- Better scalability as the project grows.
- Faster incremental rebuilds and production builds in our workload.
- Webpack-compatible ecosystem for future plugin/loader customization.
- Works cleanly with Tauri's fixed dev URL and static dist output.

Trade-off:

- Vite can feel simpler for very small apps, but we prioritize growth and build performance.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
