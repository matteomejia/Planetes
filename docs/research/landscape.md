# Existing game library unifiers

*Researched 2026-11-07. Items marked **(verify)** come from memory or secondary sources and should be checked before
we rely on them.*

## Playnite: the reference point

- **What it is:** Free, open-source (MIT) Windows library manager by Josef Nemec. It's the de facto standard for
  "one library for every launcher".
- **Stack:** C#/.NET with WPF (desktop and fullscreen modes, XAML themes), and a LiteDB local database. Playnite 11 is
  a rewrite moving to **Avalonia**, mainly to get **Linux**. It's being developed in a private repo until beta, and the
  current public release is 10.63 (Oct 2026). Pull requests to the public repo are paused during the rewrite. SDK
  11.0.0 pre-release packages exist, so the plugin API is changing.
- **Model:** Everything is a plugin.
  - *Library plugins* return games (`GetGames`) and supply install, uninstall and play actions per game. There are
    official ones for Steam, Epic, GOG, Battle.net, EA, Ubisoft, Xbox, itch.io, Amazon, Humble, and others.
  - *Metadata plugins* fill fields (IGDB is the default provider, plus store pages).
  - *Generic plugins* hook events and add UI. They're written in .NET, with PowerShell and IronPython scripting.
- **Auth:** Library plugins log in through an **embedded web view** (CEF) and store only the session cookies or tokens.
  Everything stays local.
- **Features people rely on:**
  - Playtime tracking (it monitors the game's process tree or install dir after launch)
  - Fullscreen controller mode
  - Emulator support (ROM scanning, emulator profiles)
  - Tags, categories, filters and saved filter presets
  - Duplicate merging across stores
  - Custom covers and backgrounds
  - Huge theme and extension ecosystem (HowLongToBeat, SuccessStory achievements, Duplicate Hider, …)
- **Known pain points (our opportunity):**
  - Windows-only until 11 ships
  - WPF performance with large libraries and heavy themes
  - Store integrations break when stores change their private APIs
  - Fiddly first-run setup (API keys for Steam, logins)
  - Dated feel

## GOG Galaxy 2.0

- Closed-source client with **open-source Python integration plugins** built on GOG's Galaxy SDK. It unifies libraries,
  achievements, playtime and friends.
- In 2019 only GOG and Xbox were official. Everything else was community-built, and many community integrations have
  since broken or gone unmaintained. Galaxy development itself has slowed **(verify current state)**.
- **Lesson:** a plugin model only works if someone maintains the plugins. Community plugins rot when stores change
  APIs.
- Stores data in a local SQLite file (`galaxy-2.0.db`), which other tools read for import.

## Heroic Games Launcher

- Open source (GPLv3), **Electron + React**, runs on Linux, Windows and macOS. Very popular on Steam Deck.
- Doesn't talk to stores directly. It wraps **CLI helpers**:
  - **Legendary** (Python): Epic auth, list, download, launch. Heroic uses a fork.
  - **gogdl** (Python, maintained by the Heroic team): GOG
  - **Nile** (Python): Amazon Games
- Handles Wine/Proton prefixes, cloud saves (Epic), sideloaded games.
- **Lesson:** separating "store protocol" from "launcher UI" works well. We could reuse Legendary, gogdl and Nile as
  subprocesses at first, which avoids re-implementing Epic/GOG/Amazon download protocols. The downside is bundling
  Python binaries; they ship PyInstaller builds.

## Others

| Project | Notes |
|---|---|
| **Lutris** (Python/GTK, Linux) | "Runners" (Wine, emulators, native) plus community install scripts. Imports Steam, GOG, Epic (via its own Epic integration), and more. Strongest in Linux and Wine. |
| **Rare** | Qt GUI on top of Legendary for Epic. |
| **LaunchBox / Big Box** | Proprietary, emulation-first, polished controller UI (Big Box is paid). Sets the bar for "couch mode". |
| **Steam itself** | "Add a non-Steam game", plus **Steam ROM Manager** to put everything into Steam's Big Picture. Many Deck users unify *inside* Steam. |
| **GameAtlas, Tako Launcher, Porta, Aether, …** | Small, recent open-source unifiers (their GitHub PRs show up when you search Battle.net/EA detection). Handy reference code for detection logic. |
| **Hydra Launcher** | Avoid as a reference: it's built around repack/torrent sources. |

## What this means for us

1. **The core value is reliability of integrations**, not the number of them. Playnite wins because its official
   plugins are maintained. Start with few stores done really well.
2. **Detecting installed games works locally on almost every store** (manifests, registry, local DBs). **Ownership
   (games you own but haven't installed)** needs auth and usually private APIs. That's the hard, breakable part.
3. **Differentiators we could actually offer:**
   - cross-platform from day one (macOS + Windows + Linux/Deck)
   - Rune-quality motion and performance on 5k+ game libraries
   - zero-config setup that auto-detects installed launchers with no API keys
   - first-class "Continue playing" and playtime
   - a controller mode later on
4. Keep a **plugin boundary** in our own architecture (a `Store` trait), even if every store is in-tree at first.
   Out-of-process plugins (WASM or subprocess with JSON) could come later.

## Sources

- <https://playnite.net/>
- <https://github.com/JosefNemec/Playnite>
- <https://github.com/JosefNemec/PlayniteExtensions/issues/93>
- <https://tech-insider.org/gog-galaxy-vs-playnite-2026/>
- <https://www.pcgamer.com/gogs-new-app-is-the-best-way-to-merge-steam-epic-battlenet-and-everything-else>
- <https://pcgamer.com/gog-galaxy-20-aims-to-bring-games-from-all-pc-and-console-platforms-under-one-roof>
- <https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/wiki>
- <https://github.com/CommandMC/legendary>
- <https://github.com/lippdev/tako-launcher/pull/52>
- <https://github.com/GreatDanish1026/Game-Manager/wiki/Supported-Launchers>
