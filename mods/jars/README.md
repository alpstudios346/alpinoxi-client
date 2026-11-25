# Prebuilt jars

These three jars are built by `build.sh` on this machine, without Gradle and
without Loom. **Read this before using them.**

## What they are

`alpinoxi-bridge-0.2.0.jar`, `alpinoxi-hud-0.2.0.jar` and
`alpinoxi-performance-0.2.0.jar`: complete mod jars, with their `fabric.mod.json`
and language files, compiled against a JDK 21.

## What they are not

A real Fabric build **remaps the mod against Minecraft** (Loom's `remapJar`
step), so the class files refer to Minecraft's intermediary names. These jars were
compiled against the Minecraft API as source, so they reference the API directly.
Installing them in a normal Fabric profile will fail at load with a
`NoClassDefFoundError` or a mapping mismatch.

They are here so the code can be inspected and so a working jar layout exists
today. For jars you can actually play with:

```sh
cd mods/alpinoxi-hud && ./gradlew build     # needs a JDK 21 and network once
```

That produces `build/libs/alpinoxi-hud-0.2.0.jar`, which is the one to install.

## The native library is separate

All three mods need `alpinoxi_bridge.dll` beside them:

```sh
cargo build -p alpinoxi_jni --features jni-exports --release
```

Without it the mods still load and run, with their built-in defaults, and log why.
