# GoldHLE: high-level emulator for early iOS apps

GoldHLE is a fork of touchHLE, a high-level emulator for early iOS apps. It runs on modern desktop operating systems and Android, and is written in Rust.

GoldHLE uses a high-level emulation (HLE) approach rather than low-level emulation (LLE). Instead of simulating the iPhone or iPod touch hardware directly, touchHLE itself takes the place of iOS and provides its own implementations of system frameworks such as Foundation, UIKit, OpenGL ES, OpenAL, and others. The only code executed by the emulated CPU is the app binary and a small set of bundled libraries.

The project aims to run games from the early days of iOS:

- Currently supported: iPhone, iPod touch, and iPad apps for iPhone OS 2.x, iPhone OS 3.x, and iOS 4.0.x
- Longer-term goals: high-DPI (“Retina Display”) support, newer iOS 4 versions, iOS 5.x, and iOS 6.x
- Never planned: 64-bit iOS support

This does not mean all apps from these OS versions work. The majority of iPhone OS 2.x, 3.x, and 4.x apps do not currently run in GoldHLE, and the apps that do work are generally games. Support for other app types is lower priority because it is more complex and less fun. Compatibility improves gradually as contributors add support for missing features.

If you want to learn more about the history and motivation behind the project, read the original announcement on the touchHLE project page. For technical details, see the project’s in-depth articles.

Check out the website for downloads, FAQ, social media, and more:
https://touchhle.org/

## Important disclaimer

This project is not affiliated with or endorsed by Apple Inc. in any way. iPhone, iOS, iPod, iPod touch, and iPad are trademarks of Apple Inc. in the United States and other countries.

Only use GoldHLE to emulate software you have obtained legally.

## Platform support

Officially supported:
- x64 Windows
- x64 macOS
- AArch64 Android

These are the platforms with binary releases.

Probably works, but you must build it yourself:
- AArch64 macOS
- x64 Linux
- AArch64 Linux

Never planned:
- Other architectures

### Input methods

For simulated touch input, there are four options:

- Mouse or trackpad input:
  - tap/hold/drag by pressing the left mouse button
- Virtual cursor using a game controller:
  - move the cursor with the right analog stick
  - tap/hold/drag by pressing the stick or the right shoulder button
- Mapping of controller buttons or the left analog stick to specific on-screen locations
  - see `--button-to-touch=`, `--dpad-to-touch=`, and `--stick-to-touch=` in `OPTIONS_HELP.txt`
- Real touch input on devices with a touch screen

For simulated accelerometer input, there are three options:

- Tilt simulation using the left analog stick of a game controller
- Tilt simulation using a mouse
  - hold down the right mouse button
- Real accelerometer input on devices with an internal accelerometer

## Development status

This project has been in development since December 2022. It was originally created as a passion project by hikari_no_yume and later expanded by a growing number of volunteers. Since its release in February 2023, other contributors have helped improve it in their free time, and it is no longer a single-person project.

There have only been a handful of releases so far, and no promises can be made about the future. Please be patient.

In general, the supported functionality is defined by the apps being targeted: many contributors are interested in getting a particular game working, and they add support for the missing features required for that game. As a result, compatibility varies a lot between APIs. UIKit is one of the most incomplete and hacky frameworks, because most games do not use much of it. By contrast, OpenGL ES and OpenAL support are likely complete enough to cover many early apps, since games rely heavily on them.

# Usage

First, obtain GoldHLE either from a binary release or by building it yourself.

You will then need an app to run. The app compatibility database is a good starting point for identifying which versions of apps are known to work, but keep in mind that it may contain outdated or inaccurate information. Note that the app binary must be decrypted to be usable.

There are a few ways to run an app in GoldHLE.

## Special Android notes

Windows, Mac, and Linux users can skip this section.

On Android, only the graphical user interface (app picker) is available. Therefore, you must place your `.ipa` files or `.app` bundles inside the `touchHLE_apps` directory. You can only do this after running GoldHLE once.

File management can be tricky on Android due to restrictions introduced by Google in newer Android versions. One of the following methods may work:

- If you tap the “File manager” button in GoldHLE, this should open a file manager.
  - You may also be able to find GoldHLE in your device’s file manager app (often called “Files” or “Downloads”), alongside cloud storage services.
  - Some operations may be limited.
  - Files stored in this location are kept on your device.
  - On some devices, the “File manager” button opens a file manager but crashes when performing file operations. This is likely an Android bug; if it happens, clear that file manager from your recent apps list and navigate to the device’s file manager directly instead.
- If you are on an older version of Android, you may be able to access GoldHLE’s files directly by browsing to `/sdcard/Android/data/org.touchhle.android/files/touchHLE_apps`
  - Note that `/sdcard` is usually not on the SD card.
- You may be able to use ADB.
  - If you are unfamiliar with ADB, try https://yume-chan.github.io/ya-webadb/ in Google Chrome or another browser with WebUSB.
  - Your device should be connected over USB.
  - GoldHLE files can be found under:
    - `sdcard` → `Android` → `data` → `org.touchhle.android` → `files` → `touchHLE_apps`

## Graphical user interface

GoldHLE includes a built-in app picker. If you place your `.ipa` files and `.app` bundles in the `touchHLE_apps` directory, they will appear in the app picker when you run GoldHLE.

To configure options, edit the `touchHLE_options.txt` file. To see available options, check `OPTIONS_HELP.txt`.

## Command-line interface

This section does not apply on Android.

You can view command-line usage by passing the `--help` flag.

If you are a Windows user and unfamiliar with the command line, the following steps may help:

1. Move the `.ipa` file or `.app` bundle to the same folder as `touchHLE.exe`
2. Hold Shift and right-click the empty space in the folder window
3. Click “Open with PowerShell”
4. Type:
   `.\touchHLE.exe "YourAppNameHere.ipa"`
   or
   `.\touchHLE.exe "YourAppNameHere.app"`
5. Press Enter.
6. If you want to specify options, add a space after the app name (outside the quotes) and then type the options separated by spaces

## Local multiplayer support

GoldHLE provides limited support for local multiplayer over Wi-Fi in some games. At the time of writing, it is supported in Asphalt 4 and N.O.V.A.

Real iOS devices may also join or host games.

Usage:
1. Install GoldHLE on two or more devices connected to the same Wi-Fi network
2. Important: make sure GoldHLE is allowed through your OS firewall or network settings
3. Enable “Network access” in Quick Options or via `--allow-network-access`
4. Start or join multiplayer in the game

FAQs:
- Tunneling over the Internet or through a VPN: not officially supported, but may work
- Bluetooth: not supported

Known issues:
- On macOS, you may need to launch GoldHLE from a terminal, because the OS may block network connections otherwise

## Other notes

Any data saved by the app, such as saved games, is stored in the `touchHLE_sandbox` folder.

If the emulator crashes almost immediately while running a known-working game, check whether you have overlays enabled, such as the Steam overlay, Discord overlay, or RivaTuner Statistics Server. These tools inject themselves into other apps and games and do not always clean up after themselves, which can break GoldHLE. This is not our fault. Currently, RivaTuner Statistics Server is the only known problematic overlay.

# Building and contributing

See the `CONTRIBUTING.md` file in the repo if you want to contribute. If you only want to build GoldHLE, see `dev-docs/building.md`.

# License

GoldHLE © 2023–2026 GoldHLE project contributors.

The source code of GoldHLE itself (not its dependencies) is licensed under the Mozilla Public License, version 2.0.

Due to license compatibility concerns, binaries are distributed under version 3 or later of the GNU General Public License.

For a best-effort listing of all licenses for dependencies, build GoldHLE and pass the `--copyright` flag when running it, or click the “Copyright info” button in the app picker.

Please note that different licensing terms apply to the bundled dynamic libraries in `touchHLE_dylibs/` and fonts in `touchHLE_fonts/`. Please consult the relevant directories for more information.

# Thanks

We stand on the shoulders of giants. Thank you to:

- Everyone who has contributed to the project or supported its contributors financially
- The authors and contributors of the libraries used by this project:
  - dynarmic
  - rust-macho
  - SDL
  - rust-sdl2
  - stb_image
  - Imagination Technologies’ PVRTC decompressor
  - openal-soft
  - hound
  - Symphonia
  - RustType
  - the Liberation fonts
  - the Noto CJK fonts
  - rust-plist
  - nibarchive
  - quick-xml
  - gl-rs
  - cargo-license
  - cc-rs
  - cmake-rs
  - cargo-ndk
  - cargo-ndk-android-gradle
  - md-5 and sha1
  - encoding_rs
  - corosensei
  - uuid
  - the Rust standard library
- The Skyline emulator project (RIP), for writing the boilerplate needed to replace file management on newer Android versions
- The Rust project generally
- The many people who have documented the iOS platform, officially or otherwise
- The iOS hacking and jailbreaking community
- The Free Software Foundation, for making libgcc and libstdc++ copyleft and therefore saving this project from ABI hell
- The National Security Agency of the United States of America, for Ghidra
- GerritForge for providing free Gerrit hosting
- The many contributors to Gerrit
- Friends and supporters who took an interest in the project and gave suggestions and encouragement
- Developers of early iOS apps, whose work made this project possible
- Apple, and NeXT before them, for creating such a fascinating platform
