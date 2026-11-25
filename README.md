<p align="center">
  <img src="assets/BannerForRustAlpinoxiClient.png" alt="Alpinoxi Client" width="100%">
</p>

<h1 align="center">Alpinoxi Client</h1>

<p align="center">A stylish, lightweight Minecraft Java Edition launcher written in Rust.</p>

Alpinoxi Client is a fresh, open-source Rust workspace maintained by **AlpinoxiGD**. It uses a native `egui` desktop interface rather than Electron, so the launcher stays separate from—and lightweight beside—the Minecraft Java process.

## What exists now

- Native Alpinoxi launcher shell with a dark blue, glass-inspired dashboard
- Account model for Microsoft, Ely.by and Offline profiles
- RAM profiles with explicit limits: Balanced, Performance and Custom
- An account screen which makes each provider’s status clear
- Security-first auth contract: no password collection and no token logging

## What comes next

The current UI is a foundation, not a finished Minecraft launcher. The next milestones are official Microsoft OAuth, Ely.by OAuth using client ID `alpinoxi-client`, secure OS credential storage, instance installation/launching, skin wardrobe UI, HUD settings, social features, and a licence-aware mod/resource-pack catalogue.

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

Şu an özgün launcher arayüzü, Microsoft/Ely.by/Offline hesap modeli ve RAM profilleri hazırdır. Gerçek Microsoft ve Ely.by girişleri, token kasası, Minecraft kurulum/başlatma, skin arayüzü, HUD, arkadaşlık/sohbet ve mod-kaynak paketi kataloğu sıradaki aşamalardır. Offline hesap yalnızca bunu açıkça kabul eden sunucularda kullanılabilir.

## İlham ve atıf

Bu client/başlatıcı fikrinin Polyfrost/OneClient projesinden ilham aldığını, bazı özellik fikirlerinin de Polyfrost'un YouTube içeriklerinden geldiğini unutmayınız. Alpinoxi'nin mevcut kaynak kodu bağımsız olarak yazılmıştır; bu ifade bir ortaklık veya onay anlamına gelmez.
