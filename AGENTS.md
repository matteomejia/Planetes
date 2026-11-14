# Planetes

A minimal quick launcher for installed games (Steam, Epic, GOG, Battle.net, …), built in Rust with
[Rune](https://github.com/anomalyco/rune), Anomaly's pre-release GPU UI framework. Apps pin a rev of Rune's `dev` branch.

Status: frameless, neutral/blue Rune launcher (redesigned 2026-11-07) running in background mode (tray, configurable
global hotkey in `%APPDATA%\Planetes\settings.json`, hide on focus loss, optional autostart), background scanning,
local game icons, last played and Steam playtime on each row, Steam type filtering and Steam/Epic launch handoff
implemented. Velopack installer and self-updates from GitHub Releases added 2026-11-08; releases need the .NET 8 SDK
and `vpk` 1.2.161. Read `README.md` for commands and scanner limitations, then `docs/direction.md`, `docs/research/`
and `docs/windows-build-check.md`.

The user has validated successful startup of both Steam and Epic games on this desktop.

## Goal

Ship a good public v1.0 that wins users, stars and downloads. **Standalone app first, then a PowerToys Command
Palette extension** reusing the same scanner. Main competitor: [Ludex](https://github.com/Leocadio94/ludex) (MIT, C#;
Steam/GOG/Epic/Xbox; Flow Launcher, Raycast and source-only Command Palette builds; save-folder focus). Compete on
easy installation, store coverage, launch reliability and speed rather than on save-folder features.

## Decisions

- **Minimal launcher:** open, search, select, launch, leave. A compact results list is the UI, with a preview
  panel beside it (860×520 window, user decision 2026-11-08): the selected game's local key art and logo, last
  played and playtime.
- **Quiet, non-generic design:** neutral greys, one blue accent (caret, focus, selection), dense single-line rows
  with store labels, plain status line, no decorative gradients/glows/keycaps, no title bar. The art's fade into
  the panel is the only gradient. Bundled Rune font for now. Acrylic is deferred. Verify visual changes on Windows.
- **Art:** Steam's own `appcache/librarycache` files. Epic art is downloaded once from Epic's HTTPS CDN using the
  links in its local `catcache.bin` (user decision 2026-11-08), into `%LOCALAPPDATA%\Planetes\art`;
  `"download_art": false` stops it. No third-party art services. Without art, a game shows its icon blurred.
- **Defer library-browser features:** cover grids, detail pages, collections and playtime tracking.
- **Windows first, macOS with Steam.** macOS was dropped on 2026-11-08 and brought back on 2026-11-13 (user
  decision): the same Rune UI as Windows, no macOS-only UI (Liquid Glass deferred), Steam only for now, run from
  source (no packaged release or self-updates yet). CI checks it on `macos-latest`. Platform code lives in
  `app/src/platform.rs` behind `cfg`; keep both platforms compiling.
- **Installed games only**: no store logins or ownership APIs.
- **No controller/gamepad support yet.**
- **No required API keys.** Artwork services, including optional SteamGridDB, and IGDB are deferred.
- **License: MIT.** Playnite's MIT code may be reused with its notice. Don't copy GPL code (e.g. Heroic).

## Next steps

1. v0.1.0 (background mode, Steam and Epic) was published on 2026-11-08 after the user tested it, tray menu
   included; v0.2.0 adds the Velopack installer and self-updates. Default hotkey Alt+Space; PowerToys holds it on
   this desktop, so the user sets another. Next (user decision 2026-11-08): the Command Palette extension, for
   visibility, then more stores (GOG first). v1.0 scope is in `docs/direction.md`. The repo is public (made so
   2026-11-08); draft each release and publish only with the owner's approval. See `docs/releasing.md`.
2. Release startup/search have been measured with the opt-in native benchmark; avoid adding caching without evidence.
3. Add stores incrementally while preserving the minimal search-and-launch interaction.
4. Command Palette extension over the same scanner, now ahead of the remaining v1.0 items.
