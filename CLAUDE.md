# Planetes

A desktop app that unifies game libraries (Steam, Epic, GOG, Battle.net, …), built in Rust with
[Rune](https://github.com/anomalyco/rune), Anomaly's pre-release GPU UI framework. Apps pin a rev of Rune's `dev` branch.

Status: research done, no code yet. Read `docs/direction.md` first, then `docs/research/`.

## Decisions

- **Windows is the primary platform.** Keep macOS compiling, but features and testing target Windows.
- **v0.1 shows installed games only**: no store logins or ownership APIs.
- **No controller/gamepad support yet.**
- **No required API keys in v0.1.** SteamGridDB is an optional bring-your-own key. IGDB is deferred.
- **License: MIT.** Playnite's MIT code may be reused with its notice. Don't copy GPL code (e.g. Heroic).

## Next steps

1. Build Rune's `examples/hello` and `kit-gallery` on Windows at a pinned rev. Note any issues (this is the biggest
   risk check).
2. Set up the Cargo workspace (`license = "MIT"`) as sketched in `docs/direction.md`.
3. CLI spike: detect installed Steam and Epic games on Windows and print JSON. Verify the paths marked **(verify)** in
   `docs/research/store-integrations.md` against the real installs.
4. Spike a 5,000-cover virtualised grid in Rune and measure scroll and memory.
