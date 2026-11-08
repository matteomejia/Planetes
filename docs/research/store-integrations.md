# Store integrations

*Researched 2026-11-07. Paths, endpoints and URI formats marked **(verify)** come from memory or from other projects'
code. Check each one against a real install before building on it. Stores change these without notice.*

## Windows detection spike (2026-11-07)

Implemented in `crates/stores` with a JSON CLI in `crates/cli`. Verified the Steam registry entry, modern
`libraryfolders.vdf` and app manifests across two local libraries, and both Epic paths listed below against real
installs. The scanner uses Epic's `.item` files, not the installation summary. Fortnite's separate content manifests
have `bIsApplication=false`, `bIsExecutable=false`, hidden categories and no launch executable; these are excluded.

Steam soundtrack/bonus-content manifests can resolve under `steamapps/music` instead of `steamapps/common`. These
are skipped when there is no matching common-directory install. Game-type metadata is still needed to exclude
software and shared redistributables. The launcher implementation now reads local `appinfo.vdf` types for this filter,
with synthetic v40/v41 fixtures and fallback discovery when the cache is missing or corrupt.

Steam launch URIs were checked against desktop game shortcuts. Epic uses the documented namespace, catalog item ID
and artifact/app name combination from its manifests, percent-encoded into the URI. See Epic's
[Protocol Activation guide](https://assets-unreal2-epic-prod-us2.s3.dualstack.us-east-1.amazonaws.com/original/4X/f/c/4/fc4342c50e1a0abf34943e019ee90bacccc62950.pdf).
Windows handoff uses protocol-association lookup and ShellExecuteExW with COM initialization and synchronous
activation, reporting errors in Planetes. The user has validated successful Steam and Epic startup. Playtime and
non-Windows paths remain unverified.

Local icon discovery prefers Steam's `common.clienticon` hash (falling back to `common.icon`) under `steam/games`.
Only valid 40-character hex hashes and existing files are accepted. Epic uses its manifest's existing relative
launch executable as the icon source, without executing it. Rune extracts the icons on its image workers.

The metadata reader uses [steam-vdf-parser](https://github.com/mexus/steam-vdf-parser) 0.1.2. It pads an in-memory
copy to accommodate that parser's full-header check before the four-byte EOF sentinel, and catches parser panics
so a corrupt optional cache cannot stop discovery. It never modifies Steam's cache.

See `README.md` for scan rules, path overrides, warning handling and validation commands.

For every store there are three separate questions:

1. **Installed**: what's on disk? Usually readable locally, with no auth.
2. **Owned**: what's in the account? Needs auth, often through private APIs.
3. **Launch / install / uninstall**: usually done by handing a URI or CLI to the store's own client.

## Overview

| Store | Installed (local) | Owned (account) | Launch | Platforms | Difficulty |
|---|---|---|---|---|---|
| **Steam** | `libraryfolders.vdf` + `appmanifest_*.acf` | Official Web API `GetOwnedGames` (API key + SteamID, or a public profile) | `steam://rungameid/<appid>` | Win, Mac, Linux | Easy |
| **Epic** | `LauncherInstalled.dat` + `Manifests/*.item` (JSON) | Private API via OAuth (Legendary's flow), or the `egs-api` crate | `com.epicgames.launcher://apps/<ns>%3A<id>%3A<app>?action=launch&silent=true` **(verify)** | Win, Mac | Medium |
| **GOG** | Registry `GOG.com\Games\<id>`, `goggame-<id>.info` in install dir | Semi-public `embed.gog.com` + Galaxy OAuth (gogdl's flow) | `goggame-*.info` `playTasks` or `GalaxyClient /command=runGame /gameId=<id>` | Win, Mac (+Linux via gogdl/Wine) | Medium |
| **Battle.net** | Uninstall registry entries; `ProgramData\Battle.net\Agent\product.db` (protobuf) | No API. Playnite scrapes the account web pages with a session. | `Battle.net.exe --exec="launch <CODE>"` or `battlenet://<CODE>` **(verify)** | Win, Mac | Medium–Hard |
| **EA app** | Registry / `ProgramData\EA Desktop\` install data **(verify)** | Private GraphQL with web session (Playnite does this) | `link2ea://launchgame/<offerId>` **(verify)** | Win | Hard |
| **Ubisoft Connect** | `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\<id>` **(verify)** | Limited. Local cache files. | `uplay://launch/<id>/0` | Win | Medium |
| **Xbox / Game Pass (PC)** | UWP/MSIX package enumeration (`PackageManager`), `.GamingRoot` on drives **(verify)** | Xbox Live title-history API with Microsoft OAuth | `shell:AppsFolder\<PFN>!<AppId>` | Win | Hard |
| **Amazon Games** | Local SQLite in the Amazon Games app data **(verify)** | Nile's flow | `amazon-games://play/<id>` **(verify)** | Win | Medium |
| **itch.io** | itch app's `butler` DB | Official API with an API key (owned keys) | direct exe | Win, Mac, Linux | Easy–Medium |
| **Emulation / manual** | User-pointed folders and ROM scan | n/a | emulator CLI templates | all | Easy, but lots of UX |

## Steam (start here)

- **Find Steam:**
  - Windows: registry `HKCU\Software\Valve\Steam\SteamPath`
  - macOS: `~/Library/Application Support/Steam`
  - Linux: `~/.steam/steam` or `~/.local/share/Steam`, and Flatpak `~/.var/app/com.valvesoftware.Steam/...`
- **Installed:** `steamapps/libraryfolders.vdf` lists library roots. Each root's `steamapps/appmanifest_<id>.acf` has
  name, `installdir`, size, `StateFlags`, `LastPlayed`, `LastUpdated`.
- **Owned:**
  - The `IPlayerService/GetOwnedGames` Web API with `include_appinfo=1` returns name, icon hash and **playtime**. It
    needs the user's Web API key (they create it at steamcommunity.com/dev/apikey) plus their SteamID64, or "game
    details" set to public.
  - Locally, `userdata/<id>/config/localconfig.vdf` has per-app `LastPlayed` and `Playtime`. `appcache/appinfo.vdf`
    (binary) has metadata for every known app.
  - The Web API key is a setup speed bump. Playnite asks for one. We could start without it (local only) and upgrade.
- **Non-Steam shortcuts:** `userdata/<id>/config/shortcuts.vdf` (binary). Useful for import, and later for *exporting*
  our library into Steam for Deck users.
- **Artwork:** Steam CDN library assets per appid (`library_600x900.jpg`, `library_hero.jpg`, `logo.png`,
  `header.jpg`), plus the local `appcache/librarycache/`. The CDN URL format **(verify)** has changed in recent years.
- **Rust crates:**
  - `steamlocate` (1.x stable, 2.0 beta) to find Steam and its libraries
  - `keyvalues-parser` / `keyvalues-serde` for text VDF
  - `steam-vdf-parser` for binary VDF (`shortcuts.vdf`, `appinfo.vdf`)
  - `steam-rs` for the Web API
  - Note: the `vdf` crate is something unrelated.

## Epic Games Store

- **Installed (Windows):**
  - `C:\ProgramData\Epic\UnrealEngineLauncher\LauncherInstalled.dat` (JSON list of installs)
  - `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` (JSON per game: `DisplayName`, `InstallLocation`,
    `LaunchExecutable`, `CatalogNamespace`, `CatalogItemId`, `AppName`) **(verify paths)**
  - On macOS the equivalent files live under `~/Library/Application Support/Epic/` **(verify)**.
- **Owned:** OAuth through Epic's account service, using the launcher's client credentials as Legendary does. The user
  logs in in a browser, copies an authorization code, and the app exchanges it. Then it calls the library service and
  the catalog for titles and images. The `egs-api` Rust crate implements auth, library listing (paginated), catalog and
  manifests.
- **Caveats:**
  - This is an unofficial API using Epic's own launcher client ID. It works and is widely used (Heroic, Playnite,
    Legendary), but it's against ToS in spirit and could break.
  - Prefer launching through the Epic launcher URI, so DRM and ownership checks happen there.

## GOG

- **Installed:**
  - Windows registry `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\<gameId>` (`gameName`, `path`, `exe`)
  - The `goggame-<id>.info` JSON in each install dir has `playTasks`
  - GOG Galaxy's `C:\ProgramData\GOG.com\Galaxy\storage\galaxy-2.0.db` (SQLite) holds *everything* Galaxy knows,
    including other stores it integrates. Read it opportunistically.
- **Owned:** OAuth via `auth.gog.com` using Galaxy's public client ID (gogdl and Heroic do this), then
  `embed.gog.com/user/data/games` and the products API for metadata. The login redirects to
  `embed.gog.com/on_login_success?code=...`, so without a webview the user has to paste the final URL or code.
- **DRM-free:** games can run straight from `playTasks` without Galaxy. That's a nice differentiator.

## Battle.net

- **Installed:** Uninstall registry entries (DisplayName, InstallLocation; reliable for installed games). Optionally
  decode `C:\ProgramData\Battle.net\Agent\product.db` (protobuf; Playnite issue #93, and Porta/Tako/Fenrir PRs have
  small decoders). Map product codes to titles with a table (`WoW`, `Pro` = Overwatch, `D4`, `Fen`, `OSI`, …).
  Playnite's extension has one under MIT.
- **Owned:** There's no API. Playnite scrapes `account.battle.net` with a logged-in session. Since Blizzard's catalog
  is small, the realistic approach is to treat it as **installed-only plus a static catalog**.
- **Launch:** must go through the Battle.net client (it handles auth). Be careful not to mistake the running client
  process for the running game.

## EA app, Ubisoft Connect, Xbox, Amazon

- **EA app:** detection has been brittle across the Origin → EA app migration. Start with installed-only from the
  registry or install manifests. Ownership via EA's private GraphQL needs a web session and breaks often.
- **Ubisoft Connect:** install registry keys and the `uplay://` launch URI are stable. Ownership is limited.
- **Xbox / Game Pass PC:**
  - Games are MSIX packages in protected `WindowsApps` / `XboxGames` folders. Enumerate them with the WinRT
    `PackageManager` (the `windows` crate) and filter for games. **(verify filter approach, e.g.
    `MicrosoftGame.config` presence)**
  - Ownership and Game Pass catalog through Xbox Live APIs with Microsoft OAuth (MSAL). That's doable but heavy.
- **Amazon Games:** Nile (Heroic) documents the auth and library flow. The local app keeps SQLite DBs of installs.

## Authentication without a webview

Rune has no embedded browser. Options, best first:

1. **System browser + loopback/deep-link redirect.** Works only where we control the redirect URI, which with
   borrowed client IDs we usually don't.
2. **System browser + "paste the code/URL" box.** This is what Legendary does for Epic and is common for GOG. It's
   ugly but robust. Rune's text input plus clipboard handling make it tolerable, and we can polish it ("we detected a
   GOG code on your clipboard").
3. **A small `wry` (Tauri's webview) window** opened only for login, which captures the redirect and cookies. It's the
   best UX and what Playnite effectively does, but it means a second windowing stack alongside winit. Worth a spike.
4. **Read existing launcher sessions.** For example, reuse Legendary's `user.json` if present. That's fragile and
   touches other apps' credentials, so only do it with explicit opt-in.

Store tokens in the OS keychain (`keyring` crate), never in plain config. Never ask for store passwords.

## Launching and playtime tracking

- Launch via the store URI or CLI, then watch processes (`sysinfo` crate). Watch for exes under the game's install dir.
  Don't use the launcher PID, because launchers exit, re-spawn and run anti-cheat bootstrappers.
- Record sessions locally (start, end, duration). Merge with store-reported playtime (Steam) carefully: show both, or
  take the max.
- On Linux/Deck, handle Proton/Wine launches: Steam games go through `steam://`, and others through `umu-launcher` or
  Wine. **(Later.)**

## Legal and ToS notes

- Reading local files and launching via public URIs is uncontroversial.
- Unofficial APIs with borrowed client IDs (Epic, GOG, Amazon) are grey-area. Playnite and Heroic have done it openly
  for years. Keep it read-only, rate-limited, local-only, and document it.
- Don't redistribute store artwork. Fetch it at runtime and cache it locally.

## Sources

- <https://partner.steamgames.com/doc/webapi/IPlayerService>
- <https://docs.rs/crate/steam-rs/latest/source/src/player_service/get_owned_games.rs>
- <https://www.npmjs.com/package/get-installed-steam-apps>
- <https://docs.rs/steamlocate/latest/settings.html>
- <https://lib.rs/crates/steam-vdf-parser>
- <https://rust-digger.code-maven.com/crates/keyvalues-serde>
- <https://docs.rs/crate/egs-api/latest>
- <https://github.com/CommandMC/legendary>
- <https://github.com/JosefNemec/PlayniteExtensions/issues/93>
- <https://github.com/stephenstirling/porta/pull/8>
- <https://github.com/lippdev/tako-launcher/pull/52>
- <https://github.com/xerktech/fenrir/pull/186>
- <https://github.com/ryanrn/aether/pull/27>
- <https://github.com/GreatDanish1026/Game-Manager/wiki/Game-Detection>
- <https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/wiki>
