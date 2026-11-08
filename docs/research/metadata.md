# Metadata, artwork and cross-store identity

*Researched 2026-11-07. Items marked **(verify)** come from memory.*

## Providers

| Source | Gives | Access | Notes |
|---|---|---|---|
| **IGDB** (Twitch-owned) | Canonical game records, genres, release dates, companies, covers, screenshots, **external IDs (Steam, Epic, GOG, …)** | Free. Twitch dev app → client-credentials token. Rate limit about **4 req/s**. | Query language "Apicalypse". The external IDs make it our best tool for cross-store matching. Rust: `igdb-atlas` and others. |
| **SteamGridDB** | Community grids (covers), heroes, logos, icons, in many styles | Free API key (log in with Steam → preferences → API) **(verify rate limits)** | The de facto source for good covers, used by Playnite, Steam ROM Manager and Heroic. Lookups by Steam appid or by name. |
| **Steam store / CDN** | Descriptions, screenshots, library art for Steam apps | `store.steampowered.com/api/appdetails` (unofficial, rate-limited) | Great quality for Steam titles. |
| **Store catalogs** (Epic, GOG) | Descriptions and key art for their own titles | Comes back with the library calls | Fine as a fallback. |
| **HowLongToBeat** | Completion times | Unofficial scraping | Popular Playnite extension. Later. |
| **PCGamingWiki** | Save locations, fixes, store IDs | Cargo API, open | Nice for save-backup features. Later. |

## Identity: merging the same game across stores

A user may own *Hades* on Steam and Epic. We want one tile with two "sources".

1. Each store returns `(store, store_game_id)`, which is the primary key of a **Source** row.
2. A **Game** row groups Sources. Match automatically by:
   1. IGDB `external_games` (Steam appid / GOG id / Epic id → IGDB game id). This is the most reliable.
   2. Normalised title (lowercase, strip ™®, edition suffixes such as "GOTY" and "Definitive") plus release year.
3. Always let the user merge or split by hand, and remember their choice.
4. Editions and DLC are their own rabbit hole. Hide DLC by default and treat "Deluxe Edition" as the same game.

## Local data model (sketch)

SQLite (`rusqlite`), all local:

- `game(id, title, sort_title, igdb_id, cover_path, hero_path, favorite, hidden, user_notes)`
- `source(id, game_id, store, store_game_id, owned, installed, install_path, launch_uri, size_bytes, last_synced)`
- `session(id, game_id, source_id, started_at, ended_at, seconds)`
- `tag(id, name)`, `game_tag(game_id, tag_id)`, `collection(...)`
- `artwork_cache(url, path, bytes, fetched_at)`, bounded by bytes (Rune's own rule for image caches)

## Images in Rune

- Decode with Rune's `ImageCache`: PNG, JPEG, SVG, ICNS, off-thread, mipmapped.
- WebP and AVIF covers need the `image` crate, then handing Rune RGBA through `cache.rgba(w, h, bytes)`.
- Pre-resize covers to the grid's device-pixel size on download (for example 2× of 200×300). That keeps GPU memory
  flat for 5k-game libraries.

## Sources

- <https://docs.rs/crate/igdb-atlas/latest>
- <https://docs.romm.app/3.10.2/Getting-Started/Metadata-Providers/>
- Rune `crates/rune/README.md` → "Pixels" / `ImageCache`
