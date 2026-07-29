<p align="center">
  <img src="hax.png" alt="hax logo" width="600">
</p>

# hax — a minimal TUI text editor

```
  hax <filename>     Open file directly
  hax                Start with empty buffer
```

hax is a terminal-based text editor written in Rust. It fits in **267 KB** on disk, uses **~2.3 MB of RAM**, and starts in **~2 ms**.

---

## Why

I built hax because I wanted to own my editing tools — something small enough to understand completely, fast enough to never think about, and simple enough to modify when I want to change how it works.

Modern editors (VS Code, JetBrains) are hundreds of megabytes and thousands of dependencies. Even terminal editors like nano or micro are significantly larger. hax strips everything down to the essential editing loop:

1. Open a file
2. Edit text
3. Save
4. Repeat

No plugins, no LSP, no file tree watchers, no telemetry, no JavaScript runtime. Just a text buffer, a cursor, and a terminal.

---

## How it stays lightweight

| Metric | Value | Why |
|--------|-------|-----|
| Binary | **267 KB** (UPX compressed) | `opt-level = "z"`, LTO, `codegen-units = 1`, `panic = "abort"`, `strip = true`, then UPX packed |
| Uncompressed | **603 KB** | Same build flags, no UPX |
| RAM (idle) | **~2.3 MB RSS** | No background threads, no file watchers, no JS runtime |
| CPU (idle) | **~0%** | Dirty-flag render: 0 draws at idle when no events |
| Startup | **~2 ms** | No config parsing (lazy), no plugin loading |
| Dependencies | **82 total** (2 direct) | crossterm, ratatui — no proc macros, no build scripts |
| Source | **1,736 lines** of Rust | 5 files — app.rs (421), main.rs (449), ui.rs (436), config.rs (301), theme.rs (129) |
| Warnings | **0** | Clean at all build profiles |

## Performance optimizations

- **Dirty-flag render**: Only redraws the terminal when an event is processed. At idle: 0 draws, 0 bytes written.
- **cursor_byte cache**: Byte offset cached on every edit/cursor move — O(1) insert/delete instead of O(cursor_x) scan.
- **Bulk paste**: Pastes multi-line clipboard in O(n) rather than O(n²) char-by-char insert.

### Build flags (Cargo.toml)

```toml
[profile.release]
opt-level = "z"    # optimize for size
lto = true         # link-time optimization
codegen-units = 1  # single codegen unit for max inlining
strip = true       # strip debug symbols
panic = "abort"    # no unwind tables
```

Then packed with `upx --best` for another 2x compression.

---

## Features

- **Text editing** — insert, delete, delete-forward, newline, tab
- **File management** — open, save, save-as, new, close, rename
- **File explorer** — sidebar with directory navigation, rename, parent-dir
- **Search** — across all files in the current directory (results in overlay)
- **Command palette** — filterable list of commands (save, quit, theme, etc.)
- **6 colour themes** — Monokai, Dracula, Nord, OneDark, SolarizedDark, Gruvbox
- **Clipboard** — yank/cut/paste lines (internal buffer, no OS dependency)
- **Unicode** — CJK characters render at correct display width
- **Mouse** — click to position cursor, scroll wheel to scroll
- **Configurable keybindings** — `~/.config/hax/config`
- **Unsaved changes protection** — prompts before quitting with unsaved work
- **CLI argument** — `hax main.py` opens a file directly

---

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Q` | Quit (prompts if unsaved) |
| `Ctrl+S` | Save |
| `Ctrl+O` | File explorer |
| `Ctrl+F` | Search files |
| `Ctrl+P` | Command palette |
| `Ctrl+N` | New file |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+C` | Copy (yank) line |
| `Ctrl+X` | Cut line |
| `Ctrl+V` | Paste |
| Arrows / Home / End / PgUp / PgDn | Cursor movement |
| Delete / Backspace | Delete forward / backward |
| Enter / Tab | New line / indent |

File explorer (`Ctrl+O`): `j/k` or `Up/Down` to navigate, `Enter` to open, `r` to rename, `h` to go up a directory, `Esc` to go back.

All keybindings are configurable — see **Configuration**.

---

## Configuration

Edit `~/.config/hax/config`. All defaults are listed and commented:

```
# ~/.config/hax/config
ctrl-h = cursor_left    # Vim-style left
ctrl-j = cursor_down    # Vim-style down
```

Key names are flexible: `ctrl-q`, `Ctrl+Q`, `ctrlq`, `Ctrl q` all work.

Actions available per mode are documented in the config file itself (85 lines of comments).

---

## 100% Rust

Every line of hax is Rust — no C dependencies, no build scripts, no proc macros, no unsafe blocks, no external C libraries linked. Just safe Rust through and through. The dependency tree (82 crates) is entirely pure Rust too.

One `cargo build --release` and you get a statically linked binary that runs on any Linux machine with the same `x86_64` — no interpreters, no runtimes, no containers.

---

## Performance comparison

| Editor | Disk | RAM (idle) | Startup | Language |
|--------|------|------------|---------|----------|
| **hax** | **267 KB** | **~2.3 MB** | **~2 ms** | Rust |
| nano | ~500 KB | ~3 MB | ~20 ms | C |
| micro | ~5 MB | ~10 MB | ~50 ms | Go |
| kakoune | ~2 MB | ~5 MB | ~30 ms | C++ |
| vi | ~300 KB | ~2 MB | ~10 ms | C |
| VS Code | ~400 MB | ~200 MB | ~3 s | Electron |
| JetBrains | ~1 GB | ~1 GB | ~10 s | JVM |

hax is not a replacement for VS Code. It is a replacement for "I need to edit a file without waiting for anything."

---

## Building from source

```bash
git clone https://github.com/NullSec8/hax.git
cd hax
cargo build --release
cp target/release/hax ~/.local/bin/
```

Requires Rust 2021 edition. Dependencies: crossterm, ratatui.

---

## License

MIT — do whatever you want with it.
