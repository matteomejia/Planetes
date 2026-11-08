# Planetes

A minimal quick launcher for installed games (Steam, Epic, GOG, Battle.net, …), built in Rust with
[Rune](https://github.com/anomalyco/rune), Anomaly's pre-release GPU UI framework. Apps pin a rev of Rune's `dev` branch.

Status: visually polished compact Rune launcher, background scanning, local game icons, Steam type filtering and
Steam/Epic launch handoff implemented. Read `README.md` for commands
and scanner limitations, then `docs/direction.md`, `docs/research/` and `docs/windows-build-check.md`.

The user has validated successful startup of both Steam and Epic games on this desktop.

## Decisions

- **Minimal launcher:** open, search, select, launch, leave. A compact results list is the initial UI.
- **Modern, polished presentation:** the user wants a slightly fancy UI within the minimal workflow. Refine visual
  hierarchy, surfaces, local icons and restrained motion; keep input and launch immediate. Verify effects on Windows.
- **Defer library-browser features:** cover grids, detail pages, collections and playtime tracking. The 5,000-cover
  spike is no longer a prerequisite. PowerToys Command Palette integration is a possible future frontend only.
- **Windows is the primary platform.** Keep macOS compiling, but features and testing target Windows.
- **v0.1 shows installed games only**: no store logins or ownership APIs.
- **No controller/gamepad support yet.**
- **No required API keys in v0.1.** Artwork services, including optional SteamGridDB, and IGDB are deferred.
- **License: MIT.** Playnite's MIT code may be reused with its notice. Don't copy GPL code (e.g. Heroic).

## Next steps

1. Finish v0.1 release preparation and CI; keep GitHub releases drafted and the repo private until the owner approves
   publication and a visibility change. See `docs/releasing.md`.
2. Release startup/search have been measured with the opt-in native benchmark; avoid adding caching without evidence.
3. Add GOG/Battle.net/manual entries incrementally while preserving the minimal search-and-launch interaction.
