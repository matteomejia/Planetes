# Planetes

A minimal quick launcher for installed games (Steam, Epic, GOG, Battle.net, …), built in Rust with
[Rune](https://github.com/anomalyco/rune), Anomaly's pre-release GPU UI framework. Apps pin a rev of Rune's `dev` branch.

Status: frameless, neutral/blue Rune launcher (redesigned 2026-11-07) running in background mode (tray, configurable
global hotkey in `%APPDATA%\Planetes\settings.json`, hide on focus loss, optional autostart), background scanning,
local game icons, Steam type filtering and Steam/Epic launch handoff implemented. Read `README.md` for commands
and scanner limitations, then `docs/direction.md`, `docs/research/` and `docs/windows-build-check.md`.

The user has validated successful startup of both Steam and Epic games on this desktop.

## Goal

Ship a good public v1.0 that wins users, stars and downloads. **Standalone app first, then a PowerToys Command
Palette extension** reusing the same scanner. Main competitor: [Ludex](https://github.com/Leocadio94/ludex) (MIT, C#;
Steam/GOG/Epic/Xbox; Flow Launcher, Raycast and source-only Command Palette builds; save-folder focus). Compete on
easy installation, store coverage, launch reliability and speed rather than on save-folder features.

## Decisions

- **Minimal launcher:** open, search, select, launch, leave. A compact results list is the UI.
- **Quiet, non-generic design:** neutral greys, one blue accent (caret, focus, selection), dense single-line rows
  with store labels, plain status line, no gradients/glows/keycaps, no title bar. Bundled Rune font for now.
  Acrylic is deferred. Verify visual changes on Windows.
- **Defer library-browser features:** cover grids, detail pages, collections and playtime tracking.
- **Windows is the primary platform.** Keep macOS compiling, but features and testing target Windows.
- **Installed games only**: no store logins or ownership APIs.
- **No controller/gamepad support yet.**
- **No required API keys.** Artwork services, including optional SteamGridDB, and IGDB are deferred.
- **License: MIT.** Playnite's MIT code may be reused with its notice. Don't copy GPL code (e.g. Heroic).

## Next steps

1. Background mode is implemented (default hotkey Alt+Space; PowerToys holds it on this desktop, so the user sets
   another). Still unverified by hand: tray menu items and Start with Windows. Next: more stores. v1.0 scope is in
   `docs/direction.md`. The v0.1.0 draft was cancelled; releases restart at
   v0.1.0 or v0.2.0. Keep releases drafted and the repo private until the owner approves publication and a
   visibility change. See `docs/releasing.md`.
2. Release startup/search have been measured with the opt-in native benchmark; avoid adding caching without evidence.
3. Add stores incrementally while preserving the minimal search-and-launch interaction.
4. After the app's v1.0: a Command Palette extension over the same scanner.
