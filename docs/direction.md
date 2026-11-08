# Planetes: proposed direction

*A first proposal drawn from the research in [`research/`](research/). All of it is open for discussion.*

## Pitch

A fast, beautiful, cross-platform game library that finds every game you have (Steam, Epic, GOG, Battle.net and more)
with zero setup, and launches them through their own stores. It's built with Rune, so it feels native, smooth at
120 Hz, and handles 5k-game libraries without stutter.

## Architecture sketch

```
planetes/                       (cargo workspace)
├── crates/core/                domain model, SQLite store, merge/identity, playtime sessions
├── crates/stores/              one module per store behind a `Store` trait
│   ├── steam/  epic/  gog/  battlenet/  ubisoft/  ea/  xbox/  manual/
├── crates/metadata/            IGDB, SteamGridDB, artwork download + resize cache
├── crates/launcher/            launch via URI/CLI, process watch (sysinfo), session recording
└── app/                        Rune UI (rune + rune-kit), scenes, playground, headless goldens
```

```rust
trait Store {
    fn id(&self) -> StoreId;
    fn detect(&self) -> Option<StoreInstall>;                 // is the launcher present?
    fn installed(&self) -> Result<Vec<SourceGame>>;            // local, no auth
    fn owned(&self, auth: &Auth) -> Result<Vec<SourceGame>>;   // optional, may need login
    fn launch(&self, g: &SourceGame) -> Result<LaunchPlan>;    // URI / command + process hints
    fn install(&self, g: &SourceGame) -> Option<LaunchPlan>;   // usually "open the store's install URI"
}
```

- Store syncing runs on a background runtime (tokio) and pushes into Rune state through `AppHandle::update`. Nothing
  slow touches the UI thread, and the first frame shows the cached library from SQLite instantly.
- The UI uses `rune_kit::shell`: a sidebar (Library, Recently played, per-store, collections), a main virtualised
  cover grid or list, and an optional inspector or detail pane. Cover → detail uses Rune's matched-element flight.
- Testing: every screen gets Rune scenes with fixture libraries (empty, 12 games, 5,000 games), headless `--snap`
  goldens, and `--tree` assertions. Store parsers get fixture files copied from real installs (anonymised).

## MVP (v0.1)

1. Steam: installed and local playtime (no API key), launch via `steam://`.
2. Epic: installed via manifests, launch via URI.
3. GOG: installed via registry, `.info` files, Galaxy DB, launch via playTasks or Galaxy.
4. Battle.net: installed via registry, launch via client.
5. Manual add (any exe or app bundle).
6. Cover grid + list, search, sort, filter by store or installed, favourites, hide.
7. Artwork with no API key: Steam CDN for Steam games, store-provided or locally cached art where available, a
   generated placeholder (title + colour) otherwise. **Optional** user-supplied SteamGridDB key in Settings for
   better covers on every store.
8. Playtime sessions tracked by process watching. "Continue playing" row.

**Windows is the primary target.** macOS builds should keep compiling (Rune is most mature there, and it's handy for
UI work), but features and testing aim at Windows. Linux comes later.

## Later

- Ownership via login for Epic and GOG (paste-code flow first, `wry` login window if it's worth it)
- Ubisoft, EA, Xbox, Amazon, itch.io
- Cross-store merge UI and IGDB metadata
- Controller/couch mode: `gilrs` → focus navigation, a big-screen theme. Steam Deck.
- Export to Steam shortcuts (`shortcuts.vdf` + grid art) for Deck users
- Emulation (ROM folders + emulator profiles)
- Plugins (WASM or subprocess JSON) once the `Store` trait is stable

## Decisions (2026-11-07)

- **Windows is the primary platform.** Development can start on macOS, and testing moves to a Windows desktop.
- **v0.1 shows installed games only.** No store logins.
- **No controller support yet.**
- **No API keys required for v0.1.** Installed-only detection is all local. Artwork falls back to keyless sources.
  SteamGridDB is an optional bring-your-own key. IGDB is deferred: its Twitch client secret can't be shipped safely
  in a desktop app, so it needs either a user key or a small proxy we host, which is a decision for later.

- **License: MIT** (`LICENSE`). It matches Rune and Playnite, so we can reuse Playnite's MIT code (keeping its
  copyright notice). GPL code, such as Heroic's, can't be copied in.

## Still open

1. **Reuse vs. reimplement** (post-v0.1): shell out to Legendary/gogdl/Nile (like Heroic) for installs and downloads,
   or only ever delegate installs to the official launchers?

## Suggested first spikes

1. **Rune hello on Windows**: pin a rev, build `examples/hello` + `kit-gallery` on Windows (and macOS for
   comparison), and note issues. This is the biggest risk check, so it's worth doing early on the Windows desktop.
2. **5,000-cover virtualised grid** in Rune with real JPEGs: measure scroll and memory.
3. **Steam + Epic local detection** as a CLI that prints JSON. No UI needed.
4. **Process-watch playtime** for one Steam and one Epic game.
