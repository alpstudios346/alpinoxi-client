# Alpinoxi Client

A native **C++** launcher for Minecraft Java Edition.

![Alpinoxi Client](client/assets/banner-cpp.png)

One Windows program: no Electron, no Node, no Rust, no runtime under it. The
whole launcher is `client/`, built with CMake and MSVC into a single `.exe`.

## What it does

- **Four ways to run a version** — Vanilla, Forge, Fabric and Quilt, each
  resolved from its own official metadata. A version a loader has nothing for is
  not offered, and the reason is said out loud.
- **Three ways in** — Microsoft and Ely.by with real OAuth and PKCE, and offline
  profiles that produce the exact UUID Minecraft computes.
- **Memory that makes sense** — Aikar's G1 flags trimmed for a client, scaled to
  the heap, never more than half the machine.
- **The keys you reach for** — `Shift+Tab` opens friends and the worlds that are
  open now, `Esc` closes the HUD, `Shift+H` edits it. Every one rebindable, and a
  key already taken is refused rather than firing two actions at once.
- **An editable HUD** — drag an element where you want it, pick its corner,
  toggle each one. The layout is saved with the rest of the settings.

## Build

Needs Windows, CMake 3.20 or newer and Visual Studio 2019 or newer.

```sh
cmake -S client -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release --parallel 4
cd build && ctest --build-config Release --output-on-failure
```

The launcher lands at `build/Release/alpinoxi_client.exe`, with `assets/` beside
it. The tests are a separate executable and need no window.

Every push builds and tests it on `windows-latest`: see the `cpp` workflow.

## Layout

| Path | What it is |
| --- | --- |
| `client/src/app.h`, `app.cpp` | Every decision the launcher makes: versions, loaders, memory, accounts, sign-in, friends, worlds, keys, HUD, settings, the game's command line. No Win32, so it tests with no window. |
| `client/src/ui.h`, `ui.cpp` | The window: one blue palette, seven screens, double-buffered drawing and hit-testing. |
| `client/src/main.cpp` | The entry point, and nothing else. |
| `client/src/tests.cpp` | The test suite. |
| `client/assets/` | The icon, the tray image and the banner. |
| `site/index.html` | The project page, one self-contained file. |
| `mods/` | The Fabric mods, in Java. |
| `backups/` | The old tree, kept whole. |

## The old builds

The Electron Edition (`v0.3.1`–`v0.3.5`) and the 0.2.0 mods are **end of life**.
They stay downloadable and unchanged, and their releases say so.

## Licence

MIT. Independent project, not affiliated with Mojang, Microsoft, Ely.by, Forge,
Fabric or Quilt.
