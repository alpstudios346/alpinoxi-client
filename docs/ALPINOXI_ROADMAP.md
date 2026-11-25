# Alpinoxi roadmap

This is a development plan, not a release announcement. A feature moves to a release only after implementation, review and manual game testing.

## Launcher foundation

- Preserve the existing Rust launcher architecture and avoid Electron.
- Offer Microsoft, Ely.by (`alpinoxi-client`) and clearly labelled offline profiles.
- Keep credentials in the existing secure account store; never send passwords to an Alpinoxi service.
- Let players select safe per-instance Java memory limits rather than silently allocating excessive RAM.

## Player experience

- Add an accessible, command-free skin wardrobe screen. Microsoft and Ely.by skins must follow their provider rules; local offline skins remain local.
- Add an in-game Alpinoxi HUD editor opened by a configurable keybind, with settings saved per profile.
- Design an opt-in friends and chat service with privacy controls, blocking and no collection of server IPs. It will be a separately deployed service with its own published privacy policy before public use.
- Build a searchable catalogue for mods and resource packs using official provider metadata, hashes and visible licences. A download is never silently installed or updated.

## Performance work

- Publish Alpinoxi-authored optimisations in distinct repositories/modules with source, licence, tests, reproducible benchmark scenes and compatibility notes.
- Treat Sodium, Lithium, FerriteCore, ImmediatelyFast and every other third-party project as independent work. Link to their official releases and preserve their licences; do not copy, rename or claim their code.
- Benchmark frame time, startup time and heap use against vanilla and a documented hardware/test-world matrix before making performance claims.

## Release gate

Before the first beta: verify the exact release commit, audit licence notices, test account flows without logging secrets, test a clean install and update path, provide checksums, and have a maintainer manually test the UI on each shipped platform.
