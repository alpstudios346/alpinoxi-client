<p align="center">
  <img src="assets/banner-cpp.png" alt="Alpinoxi Client" width="100%">
</p>

<h1 align="center">Alpinoxi Client</h1>

<p align="center">A stylish, lightweight Minecraft Java Edition launcher written in C++.</p>

Alpinoxi Client is a fresh, open-source project maintained by **AlpinoxiGD**. It is one **native C++ program**: a single Windows executable, with no runtime to install and no bundled browser. The window is drawn with the Win32 and GDI calls that ship with Windows.

## What exists now

Every item on this list is in the repository and covered by tests:

- **Native launcher**: `AlpinoxiClient.exe`, built from `apps/alpinoxi-cpp` with CMake and MSVC, C++20. No Electron, no Rust, no runtime beside it.
- **Three logins as peers**: Microsoft (OAuth 2.0 + PKCE, Minecraft's own client id), Ely.by (OAuth 2.0 + PKCE, client id `alpinoxi-client`), and a cracked name with the vanilla offline UUID.
- **Secure sign-in**: PKCE with S256 and a `state` value, so a code stolen from the loopback redirect is useless to anyone else. Tokens stay on the machine that signed in.
- **Skins for all three providers**: provider skins for Microsoft and Ely.by, a local PNG wardrobe for cracked players, 64×64 / 64×32 validation, classic and slim, with an honest note about who can see a skin.
- **Friends and chat for all three**: identity is `provider:uuid`, presence, requests with accept/decline, Essential-style tabs, mentions and scrollback.
- **HUD and keybinds, no commands**: Shift+H opens the HUD editor, F1 toggles the HUD, O opens friends, Y focuses chat. Launcher and game read the same settings.
- **RAM profiles**: Balanced, Performance and Custom, capped at half of system RAM, with Aikar-derived G1GC flags scaled by heap.
- **Import from another launcher**: `.minecraft`, Prism, MultiMC, CurseForge, Modrinth and Alpinoxi folders are detected, every instance is listed, and the plan is reviewed file by file before anything is copied. Worlds are copied, never moved; mods for another game version are left behind with the reason; accounts are never imported.
- **Mods and resource packs**: a licence-aware catalogue that links to official releases instead of redistributing files.
- **Alpinoxi's own Fabric mods**: a HUD mod and a performance mod, client-side only, safe on any server.
- **Performance**: the curated client-side set (Sodium, Lithium, FerriteCore, Krypton, ModernFix, Entity Culling, Indium, ImmediatelyFast, Memory Leak Fix) plus Alpinoxi Performance's own frame-time histogram, 1% lows, bottleneck diagnosis and render-distance governor.
- **Decisions separate from the window**: every rule — versions, loaders, memory, sign-in URLs, JSON reading, the game's command line — lives in `app.cpp` with no Win32 in it, and is tested by `alpinoxi-tests.exe` on every push.
- **Lunar-style layout**: a rail of screens, a top bar with the account chip and one Launch button, and a dark palette with a single accent.
- **Command-free settings**: the keybind editor takes a key press and catches conflicts with the vanilla keys; the HUD editor toggles each element and picks its anchor, with a live preview of the in-game screen.
- **One settings file**: `%APPDATA%/alpinoxi/settings.json`, plus `bridge.json` in the same folder that the in-game mods read, so the launcher's HUD preview and the HUD in game cannot disagree.
- **Loaders from 1.8**: Forge from 1.8, Fabric and Quilt from 1.14 — the floors are what each loader's own version index actually offers, not a guess.

## What comes next

- **Instance installation and launching**: downloading game files and starting the game process from the launcher.
- **Windows installer and code signing**, so the app is a normal download.
- **Modrinth publication** of the three mods, once the jars are built and tested in a real game.

Offline profiles are only for servers which explicitly allow offline-mode identities. They never bypass online-mode Microsoft/Minecraft authentication.

## Run locally

Needs CMake 3.20+ and Visual Studio 2022 (or newer) with the C++ workload.

```sh
cd apps/alpinoxi-cpp
cmake -B build
cmake --build build --config Release
build/Release/alpinoxi-client.exe      # the launcher
build/Release/alpinoxi-tests.exe       # the decisions
```

The tests are the point of the split: they run in a console with no window, so a
rule about a version, a loader, a UUID or a command line is checked in
milliseconds.

### Quality checks

```sh
cmake --build build --config Release      # /W4 /permissive-, warnings are read
build/Release/alpinoxi-tests.exe          # every decision
```

## Project layout

| Path | Purpose |
| --- | --- |
| `apps/alpinoxi-cpp` | The launcher: `app.cpp` decisions, `ui.cpp` window, `tests.cpp` |
| `apps/alpinoxi-cpp/assets` | The logo, the tray icon and the banner, byte for byte |
| `mods/` | Alpinoxi HUD, Performance and the bridge mod |
| `site/` | The project page, one self-contained HTML file |
| `docs/` | Architecture and security decisions |

## Open source

Alpinoxi Client's source is available under the [MIT License](LICENSE). Third-party mods remain the work of their respective authors: Alpinoxi links to compatible official releases and keeps their licences visible instead of copying or rebranding them.

Alpinoxi Client is not affiliated with Mojang, Microsoft or Ely.by.

## Inspiration and acknowledgement

The idea behind this client/launcher was inspired by the Polyfrost/OneClient project, and some feature ideas were inspired by Polyfrost's YouTube content. Alpinoxi's source is written independently; this acknowledgement does not imply an affiliation or endorsement.

## Türkçe

Alpinoxi Client, AlpinoxiGD tarafından sıfırdan **C++ ile** yazılan şık ve hafif bir Minecraft Java Edition başlatıcısıdır. Tek bir yerel Windows programıdır: kurulum gerektiren bir çalışma zamanı yok, gömülü tarayıcı yok. Pencere, Windows ile gelen Win32 ve GDI çağrılarıyla çizilir; bu yüzden indirme küçük, açılış anındadır.

Hazır olanlar: yerel launcher penceresi; Microsoft (OAuth 2.0 + PKCE, Minecraft'ın kendi client id'si), Ely.by (OAuth 2.0 + PKCE, client id `alpinoxi-client`) ve cracked girişi; oyunun kendi ürettiği offline UUID'ler (testlerle doğrulanmış); S256 ve `state` ile güvenli giriş; makinenin yarısını aşmayan bellek profilleri ve Aikar G1GC ayarları; 1.8'den itibaren Forge, 1.14'ten itibaren Fabric ve Quilt (yani her loader'ın kendi sürüm dizininin gerçekten sunduğu en eski sürüm); Lunar tarzı düzen; ve pencereden ayrılmış kararlar — sürüm, loader, bellek, giriş adresleri, JSON okuma ve oyunun komut satırı `app.cpp` içinde, Win32 olmadan, her push'ta test edilir.

Sıradaki aşamalar: Minecraft kurulumu ve oyunu başlatma; arkadaşlar, sohbet, skin ve HUD editörü; başka launcher'dan içe aktarma (dünyalar kopyalanır, taşınmaz; hesaplar asla içe aktarılmaz); Windows yükleyicisi ve kod imzalama. Offline hesap yalnızca bunu açıkça kabul eden sunucularda kullanılabilir.

## İlham ve atıf

Bu client/başlatıcı fikrinin Polyfrost/OneClient projesinden ilham aldığını, bazı özellik fikirlerinin de Polyfrost'un YouTube içeriklerinden geldiğini unutmayınız. Alpinoxi'nin mevcut kaynak kodu bağımsız olarak yazılmıştır; bu ifade bir ortaklık veya onay anlamına gelmez.

## Fabric mods (in `mods/`)

Alpinoxi's own mods are in this repository, under `mods/`. They are small Java
mods that **render**, and they are client-side only: they never change what the
server sees, so they are safe to use on any server.

| Mod | What it does |
| --- | --- |
| `alpinoxi-bridge` | Loads the native library and exposes it to Java |
| `alpinoxi-hud` | HUD elements, HUD editor, keybinds, friends and chat |
| `alpinoxi-performance` | Frame-time histogram, 1% lows, bottleneck diagnosis, render-distance governor |

### Build the mods

Each mod folder is a standalone Gradle project:

```sh
cd mods/alpinoxi-hud && ./gradlew build
```

The jars land in `build/libs/`.
