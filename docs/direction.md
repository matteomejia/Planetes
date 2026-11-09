# Planetes: product direction

## Purpose

A minimal, fast game launcher. Open it, find an installed game, launch it, and return to playing.
The user should spend only a few seconds in Planetes. Windows only for now; macOS support was dropped on 2026-11-08.

## Goal and roadmap

Ship a public v1.0 that wins users, stars and downloads: **the standalone app first, then a PowerToys Command
Palette extension** over the same scanner. The main competitor is [Ludex](https://github.com/Leocadio94/ludex)
(MIT, C#; Steam, GOG, Epic and Xbox for Flow Launcher, Raycast and a source-only Command Palette build, with a
save-folder focus). Planetes competes on easy installation, store coverage, launch reliability and speed.
On 2026-11-08 the user moved the Command Palette extension ahead of the remaining v1.0 items, for visibility.

v0.1.0, with background mode and Steam and Epic support, was released on 2026-11-08. v0.2.0 adds the Velopack
installer and automatic updates. Agreed v1.0 scope, delivered incrementally:

1. **Background mode (implemented 2026-11-07):** Planetes stays resident and a global hotkey summons it. The hotkey is configurable;
   Alt+Space is the default. The window hides when it loses focus or after a launch, a tray icon offers rescan,
   settings and quit, and starting with Windows is optional.
2. **More stores:** as many as each adds without complicating the implementation, one store at a time. Candidates:
   GOG, Xbox/Game Pass, Battle.net, EA app, Ubisoft Connect, Amazon Games, itch.io and manual entries.
3. **Better results:** each row shows last played and Steam's recorded playtime (done 2026-11-08). Still to do:
   recently launched first, pinned favourites, merged duplicates across stores.
4. **Per-game actions:** open install folder, open store page, uninstall through the store, from the keyboard.
5. **Distribution:** a Velopack installer and self-updating portable ZIP (released in v0.2.0; update checks are on
   by default with a setting to turn them off), later code signing, WinGet and an ARM64 build.
6. **Launch:** README with a GIF and launch posts.

## First usable milestone (done)

Build one complete Steam/Epic launch flow on top of the existing scanner:

1. A compact window with an automatically focused search field and a short results list.
2. Each result shows the title and a small store label. Use local icons when available; text is sufficient initially.
3. Typing filters results; Up/Down changes selection, Enter launches, Escape dismisses. Mouse clicks also work.
4. Launch through the official store client so its authentication and game startup flow are preserved.
5. Close the window after a successful launch handoff. If handing off fails, keep the window open with a clear error.
   A successful handoff does not prove the game itself started.
6. Load discovery off the UI thread and offer a lightweight refresh action. Keep empty, loading and failure states clear.

The first milestone uses Steam and Epic, which the CLI already detects. Add GOG, Battle.net and manual entries in
later increments without expanding the main interaction. Steam game-type classification should remove software
and shared redistributables from the default results.

These interaction details are implementation defaults for the minimal direction, rather than additional product
requirements from the user. Validate the feel with a working launch flow as presentation is refined.

## Visual direction

Minimal describes the workflow and scope. The user wants the UI to feel modern, polished and a little fancy.
Visual quality is part of the product, even though a typical session lasts only a few seconds.

The 2026-11-07 overhaul removed generic "AI app" styling: no gradients, glows, tinted monograms, keycaps or brand
header. The search field is the top of the window, results are dense single-line rows with store labels, and a
plain status line holds the count and key hints. Neutral greys carry the UI; one blue accent marks the caret, focus
and selection. Selection and hover change instantly.

Rune's pinned source supports theme palettes, shadows, rounded clipping, image/icon rendering, springs and blur.
Use these selectively, with Windows verification. In-window backdrop blur does not by itself establish desktop
translucency or native Windows acrylic support. Decorative motion must never delay typing, selection or launching;
respect reduced-motion preferences. Avoid artwork downloads and additional navigation for this polish pass.

## Scope

- Installed titles only, with local discovery and no store logins or required API keys.
- Startup, search responsiveness, keyboard navigation and reliable launch handoff take priority.
- A simple list is the initial interface. Cover galleries, detail pages, sidebars, collections, playtime tracking,
  rich metadata and artwork services are deferred. The 5,000-cover grid spike is no longer a prerequisite.
- Background mode (tray residency, a configurable global shortcut, optional autostart) is the next milestone.
- Cache discovery only when needed to keep startup immediate; avoid committing to SQLite or a background service
  before the launch flow needs them.
- No controller/gamepad support yet. Linux comes later.

## Architecture

The current Cargo workspace separates shared records (`crates/core`), discovery (`crates/stores`), JSON output
(`crates/cli`) and the Rune UI (`app`). `crates/launcher` constructs launch plans and performs OS handoff.
Discovery and launch logic remain usable independently of Rune.

Keep source identities scoped to their stores. A launch plan should retain the identifiers needed by its store;
verify Steam and Epic launch formats before implementation rather than relying on the research notes' guesses.
Do not run arbitrary executable strings from manifests through a shell.

A PowerToys Command Palette extension follows the app's v1.0. Preserve the separation between the UI and shared
discovery/launch logic so it can reuse them, for example through the CLI's JSON output.

Rune is pinned to the revision verified in [the Windows build check](windows-build-check.md). Test UI states with
fixture libraries, element-tree assertions and headless renders. Store parsers retain synthetic fixture tests;
launch handling needs tests for correct arguments and errors without launching games automatically.

## Decisions

- **Minimal quick launcher:** search, select, launch, leave. Updated after the Windows scanner spike on 2026-11-07.
- **Windows first**, while preserving portability in shared crates.
- **Installed-only, no accounts or required API keys.** IGDB and optional SteamGridDB enrichment are deferred.
- **Rust + Rune**, with the framework revision pinned in the workspace.
- **MIT license.** Reused MIT code retains its notices. Do not copy GPL implementation code.

## Next steps

The first slice is implemented: a compact Rune window, background Steam/Epic discovery, local Steam type filtering,
search, keyboard/mouse activation and shared OS launch handoff. Automated tests cover the input flow and successful/
failed handoff outcomes; Windows' missing-protocol failure is exercised without opening a game. The user has now
validated successful startup of both Steam and Epic games on this desktop. The CLI remains available separately;
`cargo run` opens the app.

The neutral/blue redesign is implemented: local icon tiles with letter fallbacks, flat selection and hover, and
distinct focus rings. The library, loading, empty and error scenes have been rendered headlessly and inspected
and checked natively on Windows.

Release startup and search have now been measured with an opt-in native benchmark. Caching remains deferred;
the library is available shortly after the first rendered frame. Windows packaging includes icon/version resources,
single-instance behavior, dependency notices and a checksum. See [Releasing](releasing.md) for the draft workflow.

1. Extend store coverage in small increments without expanding the main interaction, starting with GOG.
2. Draft each release and publish it only with the owner's approval; keep the repository private until the owner
   approves a visibility change.

The earlier broad library-browser proposal is superseded by this minimal direction. The research documents remain
references for store formats and framework capabilities, not a commitment to their full feature lists.
