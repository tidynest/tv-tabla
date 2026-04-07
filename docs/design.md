# TV-Tabla Design Spec

**Date:** 2026-04-06
**Status:** Approved
**Platform:** Windows 10/11, Linux
**Stack:** Tauri 2 + Rust backend + SolidJS frontend

---

## 1. Purpose

A desktop TV guide app for Swedish television, modelled after tvprogram.se. Primary user: a non-technical person who needs large text, simple navigation, and minimal interaction to see what's on TV. Secondary user: the developer (Linux).

## 2. Architecture

Thick Rust backend, thin SolidJS frontend, communicating via Tauri IPC commands.

### Rust Backend

Owns all I/O and persistent state:

- **XMLTV Fetcher** — downloads XML feeds from xmltv.se using `reqwest`
- **XML Parser** — parses XMLTV format using `quick-xml`
- **Data Cache** — SQLite via `rusqlite`, stores channels, programmes, favourites, settings
- **Tauri Commands** — exposes typed IPC functions the frontend calls as async functions

### SolidJS Frontend

Pure rendering layer:

- Renders the timeline grid, favourites list, week planner
- Manages UI-only state (active tab, popup, expanded sections)
- Calls Rust via `invoke()` for all data operations
- Never touches network or disk directly

### Communication

Tauri `#[tauri::command]` functions. Frontend calls them like:
```typescript
const programs = await invoke<Program[]>("get_programs", { date: "2026-04-06", hours: 4 });
```

Rust returns typed JSON. All serialisation is automatic via `serde`.

## 3. Data Source

### XMLTV from xmltv.se

- Open format, free, covers 100+ Swedish channels
- Structured XML: `<programme>` elements with start/stop, title, description, category
- Community-maintained, long-running, standard for Swedish TV tools
- Fallback: the XMLTV format is universal — alternative providers exist if xmltv.se goes down

### Caching Strategy

- **Storage:** SQLite (embedded, zero setup)
- **Cache window:** 6 hours — if data is older, fetch fresh in background
- **Stale-while-revalidate:** always show cached data immediately, update silently
- **Fetch scope:** "Now" view fetches today only. "Week" view fetches on-demand per week.
- **Manual refresh:** button available for user-triggered refresh

## 4. Database Schema

```sql
channels (
    id          TEXT PRIMARY KEY,   -- xmltv channel id
    name        TEXT NOT NULL,
    icon_url    TEXT,
    visible     BOOLEAN DEFAULT 1,
    sort_order  INTEGER
)

programs (
    id          TEXT PRIMARY KEY,   -- channel_id + start time
    channel_id  TEXT REFERENCES channels(id),
    title       TEXT NOT NULL,
    description TEXT,
    category    TEXT,
    start_time  INTEGER NOT NULL,   -- unix timestamp
    end_time    INTEGER NOT NULL,
    fetched_at  INTEGER NOT NULL    -- cache timestamp
)

favourites (
    title       TEXT NOT NULL,      -- matches by title, not programme ID
    added_at    INTEGER NOT NULL,
    PRIMARY KEY (title)
)

settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL
)
```

Favourites use title matching so that starring "Rapport" catches all future airings across channels and days.

The `settings` table stores locale preference and any future user preferences as key-value pairs.

## 5. Frontend Views

### 5.1 Tab Bar

Three tabs, always visible at the top, large click targets:

| Tab | Label (sv) | Label (en) | Label (pt) |
|-----|-----------|-----------|-----------|
| Now | Nu | Now | Agora |
| Favourites | Favoriter | Favourites | Favoritos |
| Week | Vecka | Week | Semana |

Plus a gear/cog icon for settings/channel management.

### 5.2 "Nu" (Now) — Timeline Grid

- Channels as rows (only visible ones), time as columns
- Programme blocks sized proportionally to duration
- Red vertical "now" marker line, updates every 60 seconds
- Default view: current time + 3 hours ahead
- "Visa mer" / "Show more" / "Ver mais" expander reveals rest of day
- Horizontal scroll for time, vertical scroll for channels
- Click/hover a programme block opens a popup

### 5.3 Programme Popup

Appears on click/hover over a programme block:

- Title
- Start time — end time
- Short description
- Category
- Star button to add/remove from favourites

Lightweight tooltip-style popup. Does not navigate away from the grid.

### 5.4 "Favoriter" (Favourites)

- Grouped by programme title
- Each group lists all upcoming airings across channels
- Relative dates: "idag"/"imorgon"/weekday names (localised)
- Click title to unfavourite (with confirmation dialog)
- Empty state: friendly message encouraging the user to star programmes

### 5.5 "Vecka" (Week)

- Week navigation arrows in header with week number + date range
- Navigable up to 5 weeks ahead from today's date
- Grid: channels as rows, days (Mon–Sun) as columns
- Each cell shows a condensed list of programme titles with start times for that channel/day (primetime slots prioritised if space is limited)
- Click a cell to expand inline to the full day schedule for that channel/day
- Data fetched on-demand per week

### 5.6 Settings Panel

Accessible via gear/cog icon. Slide-out panel or modal containing:

- **Language selector:** Svenska, English, Português
- **Channel visibility:** checkboxes to toggle channels on/off
- Changes persist immediately to SQLite

## 6. State Management

### Rust-side (source of truth, persisted)

- Channel list + visibility flags
- All cached programme data
- Favourites list
- Locale setting

### SolidJS-side (reactive, in-memory)

**Signals:**
- `activeTab` — "nu" | "favoriter" | "vecka"
- `currentTime` — updates every 60s, drives NOW marker
- `selectedWeek` — week offset 0–4 for Vecka view
- `dayExpanded` — whether "Visa mer" is open
- `popupProgram` — programme currently showing popup, or null
- `channelMgrOpen` — whether settings panel is visible

**Resources (async, from Rust IPC):**
- `channels()` — `invoke("get_channels")`
- `programs()` — `invoke("get_programs", { date, range })`
- `favourites()` — `invoke("get_favourites")`

No frontend state management library. SolidJS signals + resources are sufficient. Mutations trigger resource refetch for automatic UI updates.

## 7. Internationalisation (i18n)

Three locales: `sv-SE` (default), `en-GB`, `pt-BR`.

Implementation:
- Flat JSON dictionary per locale (~50 translatable strings)
- A `locale` signal stored in SQLite settings table
- A `t("key")` helper function reads from active dictionary
- Language switch takes effect immediately, no restart
- Date/time formatting respects active locale (weekday names, relative dates, week numbering)

No heavy i18n library — the string count doesn't justify one.

## 8. Error Handling

### Principles

1. Never crash, never show a blank screen
2. No technical jargon in user-facing messages (all messages in active locale)
3. Log errors to file silently for remote debugging
4. No loading indicators for background fetches — grid just updates when data arrives

### Failure Matrix

| Failure | User experience | System behaviour |
|---|---|---|
| No internet, no cache | Friendly localised message + retry button | No crash |
| No internet, has cache | Normal display + yellow "showing saved data" banner | Background retry every 5 min |
| xmltv.se down | Same as no internet | Falls back to cache, logs failure |
| Corrupt XML per channel | Other channels display normally | Bad channel shows "Data saknas" / "No data" / "Sem dados" |
| SQLite error | Generic error only if unrecoverable | Log to file |

### Log Locations

- Linux: `~/.config/tv-tabla/logs/`
- Windows: `%APPDATA%/tv-tabla/logs/`

## 9. Packaging & Distribution

### Project

- **Name:** tv-tabla
- **Location:** `~/RustroverProjects/tv-tabla/`

### Build Targets

| Platform | Output | Method |
|---|---|---|
| Windows 10/11 | `.msi` installer | Cross-compile or GitHub Actions CI |
| Linux | AppImage + `.deb` | Native build |

### App Data

Tauri's `app_data_dir()` handles platform paths automatically:
- Linux: `~/.config/tv-tabla/`
- Windows: `%APPDATA%/tv-tabla/`

### v1 Distribution

Build `.msi`, deliver to user directly (USB stick, file transfer).

### Future (v2): Auto-Update

Tauri 2 `tauri-plugin-updater` — checks a manifest URL on launch, downloads and applies updates silently. Architecture doesn't need to change; it's a plugin addition.

## 10. Rust Crate Dependencies (Expected)

| Crate | Purpose |
|---|---|
| `tauri` | Application framework |
| `reqwest` | HTTP client for XMLTV fetching |
| `quick-xml` | XMLTV parsing |
| `rusqlite` (bundled) | SQLite database |
| `serde` / `serde_json` | Serialisation for IPC |
| `chrono` | Date/time handling |
| `tokio` | Async runtime (Tauri default) |
| `log` / `env_logger` | Logging |

## 11. SolidJS Dependencies (Expected)

| Package | Purpose |
|---|---|
| `solid-js` | UI framework |
| `@tauri-apps/api` | Tauri IPC bindings |
| `typescript` | Type safety |
| `vite` | Build tool (Tauri default) |

Minimal dependency footprint. No CSS framework — custom CSS for the grid layout.

## 12. Non-Goals (v1)

- Streaming/playback integration
- Notifications/reminders for upcoming programmes
- User accounts or cloud sync
- Programme ratings or reviews
- Search functionality (future candidate)
- Auto-update mechanism (v2)
