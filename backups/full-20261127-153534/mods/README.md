# Alpinoxi mods

Three Fabric mods, all client-side, all driven by the Rust core in `packages/`.

| Folder | Mod id | Depends on |
| --- | --- | --- |
| `alpinoxi-bridge` | `alpinoxi-bridge` | — |
| `alpinoxi-hud` | `alpinoxi-hud` | `alpinoxi-bridge`, Fabric API |
| `alpinoxi-performance` | `alpinoxi-performance` | `alpinoxi-bridge` |

## Building

```sh
# 1. the native library
cargo build -p alpinoxi_jni --features jni-exports --release
# -> target/release/alpinoxi_bridge.dll  (libalpinoxi_bridge.so / .dylib)

# 2. the mods (each folder is its own Gradle project)
cd mods/alpinoxi-bridge      && ./gradlew build
cd mods/alpinoxi-hud         && ./gradlew build
cd mods/alpinoxi-performance && ./gradlew build
```

Drop the three jars and the native library into `.minecraft/mods/`. If the library
sits somewhere else, point at it with `-Dalpinoxi.bridge=C:/path/alpinoxi_bridge.dll`.

## The rule

**Java renders, Rust decides.**

* `packages/alpinoxi_core` — HUD anchors, keybind chords, memory profiles, mod list.
* `packages/alpinoxi_ingame` — chat tabs, friend book, frame statistics, diagnosis.
* `packages/alpinoxi_jni` — the JNI surface: `net.alpinoxi.bridge.Native`.

A layout bug, a keybind conflict or a wrong performance recommendation is fixed
once, in Rust, with a test, and every mod gets it. Java keeps only what must be
Java: mixins, rendering, and the game's events.

## Adding a HUD element

1. `HudElement` in `packages/alpinoxi_core/src/hud.rs` (variant, `ALL`, `label`,
   `default_anchor`).
2. `element_id()` in `packages/alpinoxi_jni/src/lib.rs`.
3. One `case` in `HudRenderer.label()`.

Three small edits, and `cargo test` covers the layout half of it.

## What the performance mod does not do

It does not patch the renderer. Sodium does that better, and two mods editing the
same pipeline is how FPS drops. It measures, it names the bottleneck, and it moves
the render distance one chunk at a time when you ask it to.
