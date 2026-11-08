# Rune: what it is and whether it suits this app

*Researched 2026-11-07 against `anomalyco/rune` `dev` @ `250597b` (cloned and read, not just its landing page).*

## Summary

Rune is Anomaly's (the OpenCode team's) MIT-licensed, GPU-rendered Rust UI framework for macOS, Windows, Linux and
the web. Apart from a few foundation crates, it builds its whole stack itself: the renderer (wgpu everywhere, plus its
own Metal backend on macOS), text (Parley, HarfRust, swash, with rasterisation tuned to match CoreText and DirectWrite),
layout (Taffy flexbox/grid), springs and animation, accessibility (AccessKit), and platform shells (winit plus custom
pieces). It is built to be **written and checked by coding agents**: every app can run headless, render PNGs, dump its
element tree as JSON and replay scripted input on a fixed virtual clock.

**Status: pre-release.** There are no versions. Apps pin a git rev of `dev`, which is fast-forward-only and never
rewritten. The project doesn't accept PRs: you file an issue, and their agents usually handle it quickly. It has about
3.6k stars and is very active (the clone above was hours old). Its flagship test is rebuilding OpenCode Desktop, and
Hex (Anomaly's dictation app) already ships Rune screens.

```toml
rune = { git = "https://github.com/anomalyco/rune", rev = "<dev rev>" }
rune-kit = { git = "https://github.com/anomalyco/rune", rev = "<same rev>" }
```

## Programming model

- Each **view function** is a plain Rust function `fn view(s: &State, cx: &mut Cx<State>) -> El`, built with
  `row()/col()/el()/text()` and chained style and layout methods. Views don't use a macro DSL.
- The **retained, keyed tree** is rebuilt from the view and diffed. Explicit `.key(..)` keeps identity stable.
- **Handlers** (`.on_press(|s: &mut State| ..)`) are queued and run between frames with `&mut State`, then the view
  re-runs. Because handlers never run during render, you can't get a `RefCell` re-entry panic.
- **Animation** comes from analytic springs on every property, using SwiftUI's `(duration, bounce)` parameters. Views
  only declare targets and the runtime animates between them. Layout changes, presence (enter and exit) and colour
  changes animate for free.
- **App hooks**: `setup` (gets a `Waker` for background threads), `poll(state, clock, tx)` before each frame,
  `on_ready` (for slow work after the first frame), and `AppHandle::update(|s, tx| ..)` from any thread.
  `rune::Latest<T>` runs latest-wins background requests (made for search-as-you-type).
- **Async**: there is no built-in async runtime. You run your own (for example tokio on a thread) and push results in
  through `AppHandle::update` or a `Waker`.

```rust
fn view(s: &State, _: &mut Cx<State>) -> El {
    let t = tok::theme(true);
    let switch = el().key("switch").role("switch").label("Lights").checked(s.on)
        .focusable(true).size(38.0, 22.0).radius(11.0)
        .fill(if s.on { tok::accent(0, true) } else { t.control_off })
        .motion(motion::SNAPPY)
        .on_press(|s: &mut State| s.on = !s.on)
        .child(el().key("knob").size(18.0, 18.0).radius(9.0).fill(t.knob)
            .absolute(if s.on { 18.0 } else { 2.0 }, 2.0));
    col().w_full().h_full().center().gap(16.0).fill(t.bg).child(switch)
}
```

## Features that matter for a game library

| Need | Rune today |
|---|---|
| Big grid or list of games (thousands) | `virtual_rows` / `.virtualize(..)` on a scroll viewport: only visible rows get layout, text and scene nodes. The plan targets 1M-row lists at ≤ 1–2 ms. Scrolling costs no view commit. **Caveat:** virtual rows are vertical-only, and "view construction still visits every row; a lazy view API is a remaining optimization." A grid has to be built as virtualised rows of N covers. |
| Cover art, hero images | `ImageCache` (PNG, JPEG, SVG, PNG-ICNS; deduped, decoded off-thread, mipmapped) and `rune::Pixels` for streamed RGBA. Caches must be bounded in bytes (their own perf rule). WebP and AVIF are **not** listed. Many artwork sources (SteamGridDB, Steam CDN) serve JPEG/PNG, but you may need to decode WebP yourself with the `image` crate and hand Rune RGBA. |
| Game executable icons | `ImageCache::app_icon` rasterises an `.exe`'s or `.ico`'s icon on Windows and a `.desktop` entry's icon on Linux. That's directly useful here. |
| Search box | `TextInput`, `Combobox` (fuzzy, virtualised), `Latest<T>` for debounced off-thread queries. |
| App frame | `rune_kit::shell`: resizable, collapsible sidebar, toolbar, optional inspector pane, keyboard-navigable sidebar list. A good fit for "Library / Steam / Epic / Collections" navigation. |
| Components | Button, IconButton, Switch, Checkbox, Radio, Select, Menu and context menus, Popover, Tooltip, HoverCard, Toast, Card, List, Accordion, Badge, Progress, Spinner, SegmentedControl, Tree, Table, Markdown, Diff. |
| Theming | Themes as JSON data with light/dark and runtime switching. `Kit::from_palette` derives a full theme from 3 seed colours, which makes user themes feasible. |
| Shared-element motion | "Matched" flights (cover → detail page hero) are built in. This kind of motion is what would make the app *feel* premium. |
| Background or tray app | `App::windowless()/keep_running()`, `AppHandle` show/hide/open, compact tray-controlled windows (you bring your own tray crate). |
| Notifications | `rune-notify`: macOS `UNUserNotificationCenter`, Windows toasts. Linux isn't supported yet. |
| Deep links | `App::on_open_url` on macOS, plus `rune::single_instance` with argv on Windows. Useful for OAuth redirects to `planetes://`. |
| File dialogs, drag and drop | Built in. |
| Accessibility | AccessKit: UIA on Windows, NSAccessibility, AT-SPI. |
| Video (trailers) | `Player` is **macOS only** (AVFoundation). Other platforms report `Failed`. |
| Gamepad / controller input | **None.** No gilrs or gamepad code in the repo. A "Big Picture" or couch mode would need your own `gilrs` thread that turns stick and D-pad input into Rune focus-navigation keys. |
| Embedded web view | **None.** Matters for store logins (see [store-integrations.md](store-integrations.md#authentication-without-a-webview)). |

## Agent and testing workflow (a real advantage for us)

- `cargo run -p app -- --snap out.png --scene <name>` renders headless, deterministically.
- `--tree` dumps the element and accessibility tree as JSON. Scripts (`tap`, `type`, `key`, `scroll`, `wheel`,
  `resize`, …) drive the app on a virtual clock.
- Scenes register named states with fixture data. The in-app **Playground** (debug builds) lists every scene.
- `Runtime::new(..)` in unit tests lets you click, step frames and assert on presented rects (see `examples/hello`).
- Hot reload: tokens and literals in ≤ 100 ms with no rebuild, and structural Rust edits via subsecond in about 1 s
  (macOS).
- Their repo ships `AGENTS.md` and `CLAUDE.md` with conventions, and the README says "give your coding agent this repo,
  start from `examples/hello`."

## Risks for *this* project

1. **Windows maturity is the big one.** PC gaming is overwhelmingly Windows, and most stores (Battle.net, EA, Ubisoft,
   Xbox) are Windows-only. Rune's centre of gravity is macOS: roughly 750 `cfg(target_os = "macos")` sites against
   about 220 for Windows in `crates/rune/src`. Several features are Mac-only (video, some window policies), the
   measured perf numbers are all Apple silicon, and the public CI workflows only build the site. Windows support is
   real (a DirectWrite-matched text mode, UIA, toasts, `single_instance`, exe icons), and OpenCode Desktop on Windows
   is a stated goal. Still, budget for filing Windows bugs upstream.
2. **No releases, no PRs.** Every bump of the pinned rev can move output (they keep a `CHANGELOG.md` of exactly those
   changes). Upstream fixes go through issues. Plan to bump deliberately and keep our own headless goldens to catch
   regressions.
3. **API churn.** It's pre-1.0 and agent-written, so expect renames. Their `AGENTS.md` lists struct literals that
   break when fields are added (`AppSpec`, `ScrollSpec`, …).
4. **Missing pieces we need**: gamepad input, a webview, WebP/AVIF decode, a lazy grid, video off macOS. All of these
   can be worked around in app code. None of them are blockers for a desktop-first MVP.
5. **Linux**: needs `libxkbcommon-dev libwayland-dev` to build. Steam Deck (gamescope, controller-only) would hit the
   gamepad gap hardest.

## Alternatives, for calibration

| Framework | Why consider it | Why not, vs. Rune |
|---|---|---|
| **Tauri / Electron** (Heroic uses Electron) | Webviews make store logins trivial, huge ecosystem | Heavy, web look and feel. Not what you're excited about. |
| **GPUI** (Zed) | Mature, cross-platform now | Rune's own research chose to start fresh because of GPUI's compositing and animation limits |
| **Slint, Iced, egui, Dioxus native** | Released and stable | Weaker motion and text quality. Rune's whole pitch is feel. |
| **Avalonia (.NET)** | It's what Playnite 11 is moving to | Not Rust |

**Verdict:** Rune fits a library browser well: big virtualised lists, image-heavy views, polish, motion, headless
testing that suits agent-driven development. The app is mostly "read local files plus HTTP, then show a beautiful grid
and launch a URI", which plays to Rune's strengths and stays away from its weak spots (complex text editing, iOS).
Go in expecting to be an early adopter on Windows.

## Sources

- Repo: <https://github.com/anomalyco/rune> (README, PLAN.md, `crates/rune/README.md`, `crates/rune-kit/README.md`, AGENTS.md, CHANGELOG.md)
- <https://github.com/anomalyco/rune/blob/dev/PLAN.md>
- <https://www.ayu-sh-kr.com/news/rune-native-ui-framework/>
- <https://daily.dev/posts/rune-a-native-gpu-rendered-rust-ui-framework-for-macos-windows-linux-and-web-xwlukkwrq>
