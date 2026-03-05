# yew-playground

Playground for Yew. Hosted at https://play.yew.rs using Google Cloud Run

## Local Development

```console
cargo r -r -p devtool
```

This opens a TUI that runs all three services simultaneously:

- **Compiler** (port 4000+): compiles user-submitted Yew apps via `trunk build`
- **Backend** (port 3000+): serves the playground API, proxies to the compiler
- **Frontend** (port 8080+): the Yew frontend served by `trunk serve`

Ports auto-increment if already in use.

### Keybindings

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch focused panel |
| `Up` / `Down` / `j` / `k` | Scroll focused panel |
| `End` | Jump to bottom |
| `r` | Restart focused service |
| `R` | Restart all services |
| `1` / `2` / `3`  | Copy panel output to clipboard |
| `q` / `Esc` / `Ctrl+C` | Quit |

Mouse hover changes focus. Scroll wheel works. `Shift+drag` for text selection. Each panel also has a clickable "Copy" button to the upper right.

### Prerequisites

- Rust nightly with `wasm32-unknown-unknown` target
- [trunk](https://trunk-rs.github.io/trunk/) (`cargo install trunk`)
- Node.js (for tailwind CSS)

### First-time Setup

Run `npm install` in the `frontend/` directory to install Node dependencies (required for Tailwind CSS and Material Design icons).
