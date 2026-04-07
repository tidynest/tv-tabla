# TV-Tabla

A desktop TV guide for Swedish television. Built with Tauri 2, Rust, and SolidJS.

TV-Tabla fetches programme schedules from [tv24.se](https://tv24.se), caches them locally in SQLite, and presents them in a clean timeline grid. Designed for people who just want to see what's on — large text, simple navigation, no account required.

<!-- TODO: Add screenshot here -->
<!-- ![TV-Tabla screenshot](docs/screenshot.png) -->

## Features

- **Now view** — horizontal timeline grid with a live "now" marker, channels as rows, programmes as time-proportional blocks
- **Favourites** — star any programme by title and see all upcoming airings across channels and days
- **Week planner** — browse schedules up to 5 weeks ahead, day-by-day per channel
- **Offline-first** — cached data displays instantly; background refresh keeps it current
- **Multilingual** — Swedish (default), English, and Portuguese
- **Channel management** — toggle channel visibility and reorder to your preference
- **Privacy-respecting** — no accounts, no tracking, all data stays on your machine

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri 2](https://v2.tauri.app/) |
| Backend | Rust |
| Frontend | [SolidJS](https://www.solidjs.com/) + TypeScript |
| Database | SQLite (embedded via rusqlite) |
| Data source | [tv24.se](https://tv24.se) |
| Build tool | Vite |

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+ and [pnpm](https://pnpm.io/)
- Linux: system dependencies for WebKitGTK — see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#linux)
- Windows: WebView2 (included in Windows 10/11)

### Development

```bash
pnpm install
pnpm tauri dev
```

### Production Build

```bash
pnpm tauri build
```

Output appears in `src-tauri/target/release/bundle/` — `.deb` and AppImage on Linux, `.msi` on Windows.

## Project Structure

```
src/                  # SolidJS frontend
  components/         # UI components (TimelineGrid, Favourites, WeekView, etc.)
  i18n/               # Translations (sv, en, pt)
  lib/                # API wrappers, state management, types
src-tauri/            # Rust backend
  src/
    commands.rs       # Tauri IPC command handlers
    db.rs             # SQLite operations
    fetcher.rs        # Schedule data fetching
    models.rs         # Data structures
    tv24.rs           # tv24.se provider
    xmltv.rs          # XMLTV parser
docs/                 # Design documentation
```

## License

[MIT](LICENSE)
