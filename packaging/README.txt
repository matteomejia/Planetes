Planetes - Windows x64

Installed with Planetes-win-Setup.exe, or extracted from Planetes-win-Portable.zip and opened
with Planetes.exe. Rust is not required.
Requires a Windows PC with a graphics driver supported by wgpu (DirectX 12 or Vulkan).
This release has been tested on Windows 11. Steam and Epic Games Launcher must be installed
for their games to be discovered and launched through their official clients.

Planetes runs in the background with a tray icon. Press Alt+Space to open it, type to find
an installed game, then use Up/Down and Enter, or click a result. Escape, the hotkey or clicking
away hides it. Ctrl+R checks your installed libraries again.

To change the hotkey, choose Edit settings in the tray menu, for example:
  { "hotkey": "ctrl+alt+space" }
The tray menu can also start Planetes with Windows, and Quit exits it.
Opening Planetes again shows the existing window instead of opening a duplicate.

No accounts or API keys are needed. Planetes checks GitHub for a new version after it starts
and then daily, and installs it the next time its window hides. To turn this off, add
"check_for_updates": false to the settings file. Epic games' art is downloaded once from Epic's
own site; add "download_art": false to turn that off.
This build is unsigned; Windows may identify its publisher as unknown.

Updates and source: https://github.com/matteomejia/Planetes
Report problems: https://github.com/matteomejia/Planetes/issues

LICENSE contains Planetes' MIT license. THIRD-PARTY-NOTICES.txt covers bundled dependencies.
BUILD-INFO.json records the version, source commit and executable SHA-256.
