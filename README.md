<p align="center">
  <img src="assets/BannerForRustAlpinoxiClient.png" alt="Alpinoxi Client" width="100%">
</p>

<h1 align="center">Alpinoxi Client</h1>

<p align="center">A stylish, lightweight Minecraft Java Edition launcher written in Rust.</p>

Alpinoxi Client is a fresh, open-source Rust workspace maintained by **AlpinoxiGD**. It uses a native `egui` desktop interface rather than Electron, so the launcher stays separate from—and lightweight beside—the Minecraft Java process.

## What exists now

Every item on this list is in the repository and covered by tests:

- **Native launcher**: `egui` dashboard, no web view, thin LTO and stripped release build.
- **Three logins as peers**: Microsoft device-code → Xbox Live → XSTS → `login_with_xbox`, Ely.by OAuth 2.0 + PKCE with client id `alpinoxi-client`, and a cracked name with the vanilla offline UUID.
- **Secure OS credential storage**: refresh tokens go to the Windows Credential Manager through `CredWriteW`/`CredReadW`. No token is ever written to a file Alpinoxi controls.
- **Skins for all three providers**: provider skins for Microsoft and Ely.by, a local PNG wardrobe for cracked players, 64×64 / 64×32 validation, classic and slim, with an honest note about who can see a skin.
- **Friends and chat for all three**: identity is `provider:uuid`, presence, requests with accept/decline, Essential-style tabs, mentions and scrollback.
- **HUD and keybinds, no commands**: Shift+H opens the HUD editor, F1 toggles the HUD, O opens friends, Y focuses chat. Launcher and game read the same settings.
- **RAM profiles**: Balanced, Performance and Custom, capped at half of system RAM, with Aikar-derived G1GC flags scaled by heap.
- **Import from another launcher**: `.minecraft`, Prism, MultiMC, CurseForge, Modrinth and Alpinoxi folders are detected, every instance is listed, and the plan is reviewed file by file before anything is copied. Worlds are copied, never moved; mods for another game version are left behind with the reason; accounts are never imported.
- **Mods and resource packs**: a licence-aware catalogue that links to official releases instead of redistributing files.
- **Alpinoxi's own Fabric mods**: a HUD mod and a performance mod, both driven by the Rust core through a JNI bridge.
- **Performance**: the curated client-side set (Sodium, Lithium, FerriteCore, Krypton, ModernFix, Entity Culling, Indium, ImmediatelyFast, Memory Leak Fix) plus Alpinoxi Performance's own frame-time histogram, 1% lows, bottleneck diagnosis and render-distance governor.

## What comes next

- **Instance installation and launching**: downloading game files and starting the game process from the launcher.
- **Windows installer and code signing**, so the app is a normal download rather than a `cargo run`.
- **Modrinth publication** of the three mods, once the jars are built and tested in a real game.

Offline profiles are only for servers which explicitly allow offline-mode identities. They never bypass online-mode Microsoft/Minecraft authentication.

## Run locally

Install the current stable Rust toolchain, then run:

```sh
cargo run -p alpinoxi_client
```

Quality checks:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Project layout

| Path | Purpose |
| --- | --- |
| `apps/alpinoxi_client` | Native desktop app and visual design system |
| `packages/alpinoxi_core` | Account, launch and memory domain models |
| `packages/alpinoxi_auth_system` | Provider-independent authentication state machine |
| `docs/` | Architecture and security decisions |

## Open source

The new Alpinoxi source is available under the [MIT License](LICENSE). Third-party mods remain the work of their respective authors: Alpinoxi will link to compatible official releases and keep their licences visible instead of copying or rebranding them.

Alpinoxi Client is not affiliated with Mojang, Microsoft or Ely.by.

## Inspiration and acknowledgement

The idea behind this client/launcher was inspired by the Polyfrost/OneClient project, and some feature ideas were inspired by Polyfrost's YouTube content. Alpinoxi's current source is written independently; this acknowledgement does not imply an affiliation or endorsement.

## Türkçe

Alpinoxi Client, AlpinoxiGD tarafından sıfırdan Rust ile yazılan şık ve hafif bir Minecraft Java Edition başlatıcısıdır. Electron kullanmaz; native arayüz sayesinde başlatıcı, Minecraft’ın Java işleminden ayrı ve düşük kaynak tüketimli kalır.

Hazır olanlar: özgün launcher arayüzü; Microsoft (device code → Xbox Live → XSTS → `login_with_xbox`), Ely.by (OAuth 2.0 + PKCE, client id `alpinoxi-client`) ve cracked girişi; Windows Kimlik Bilgisi Yöneticisi'nde tutulan refresh token'lar; üç giriş türü için de skin ve arkadaşlık/sohbet; komutsuz HUD ve keybind'ler; yarım RAM sınırlı bellek profilleri ve Aikar G1GC ayarları; başka launcher'dan (Prism, MultiMC, CurseForge, Modrinth, .minecraft) içe aktarma; lisanslara saygılı mod/kaynak paketi kataloğu; ve Rust çekirdeğiyle çalışan kendi Fabric modlarımız (HUD + performans).

Sıradaki aşamalar: Minecraft kurulumu ve oyunu başlatma, Windows yükleyicisi ve kod imzalama, modların Modrinth'te yayınlanması. Offline hesap yalnızca bunu açıkça kabul eden sunucularda kullanılabilir.

## İlham ve atıf

Bu client/başlatıcı fikrinin Polyfrost/OneClient projesinden ilham aldığını, bazı özellik fikirlerinin de Polyfrost'un YouTube içeriklerinden geldiğini unutmayınız. Alpinoxi'nin mevcut kaynak kodu bağımsız olarak yazılmıştır; bu ifade bir ortaklık veya onay anlamına gelmez.

## Alpinoxi Client 0.2 — what is now in the code

- **Three login types as peers**: Microsoft (device code flow, official Xbox → XSTS → `login_with_xbox` chain), Ely.by (OAuth 2.0 + PKCE, client id `alpinoxi-client`) and Cracked (validated name, vanilla offline UUID).
- **Skin system for all three**: provider skins for Microsoft/Ely.by, a local PNG wardrobe for cracked players, 64x64 / 64x32 validation, classic and slim, and an honest note about who can see it.
- **Friends and chat for all three**: identity is `provider:uuid`, presence (offline, launcher, in game), requests with accept/decline, tabs, mentions and history.
- **HUD and keybinds, no commands**: Shift+H opens the HUD editor, F1 toggles the HUD, O opens friends, Y opens chat. Launcher and game read the same settings.
- **Performance**: RAM plan capped at half of system RAM, Aikar-derived G1GC flags scaled by heap, and a curated client-side performance set (Sodium, Lithium, FerriteCore, Krypton, ModernFix, Entity Culling, Indium, ImmediatelyFast, Memory Leak Fix).
- **Mods and resource packs**: a catalogue that links to official releases instead of redistributing files.

Build and check:

```sh
cargo run -p alpinoxi_client
cargo test --workspace
```

The launcher itself stays small on purpose: native `egui`, no web view, `lto = "thin"`, `strip`, and no runtime of its own.

## Fabric mods (in `mods/`)

Alpinoxi's own mods are in this repository, under `mods/`. They are small Java
mods that **render**; every decision comes from the Rust crates, reached through
`packages/alpinoxi_jni`.

| Mod | What it does | Rust source of truth |
| --- | --- | --- |
| `alpinoxi-bridge` | Loads the native library and exposes it to Java | `packages/alpinoxi_jni` |
| `alpinoxi-hud` | HUD elements, HUD editor, keybinds, friends and chat | `alpinoxi_core::hud`, `alpinoxi_ingame` |
| `alpinoxi-performance` | Frame-time histogram, 1% lows, bottleneck diagnosis, render-distance governor | `alpinoxi_ingame::perf` |

### Build the native library

```sh
cargo build -p alpinoxi_jni --release
```

The result is `alpinoxi_bridge.dll` (Windows), `libalpinoxi_bridge.so` (Linux) or
`libalpinoxi_bridge.dylib` (macOS). Copy it next to the mod jars, or let the mod
find it through `-Dalpinoxi.bridge=<path>`.

### Build the mods

Each mod folder is a standalone Gradle project:

```sh
cd mods/alpinoxi-hud && ./gradlew build
```

The jars land in `build/libs/`. The mods are **client-side only**: they never
change what the server sees, so they are safe to use on any server.

### Why the split

A HUD layout, a keybind conflict, a chat rule or a render-distance decision is
logic, not drawing. Keeping it in Rust means it is covered by `cargo test`, and a
bug fix ships once to the launcher and to every mod. Java keeps only what must be
Java: mixins, rendering and the game's own events.
