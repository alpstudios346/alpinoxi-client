<p align="center">
  <img src=".github/media/RepoBanner.png" alt="Alpinoxi Client — Rust-based launcher for Minecraft" width="100%">
</p>

<p align="center">
  <strong>A lightweight, open-source Rust launcher for Minecraft Java Edition.</strong>
</p>

<p align="center">
  <a href="https://github.com/alpstudios346/alpinoxi-client/releases">Releases</a> ·
  <a href="CONTRIBUTING.md">Contributing</a> ·
  <a href="ATTRIBUTION.md">Attribution</a> ·
  <a href="docs/ALPINOXI_ROADMAP.md">Roadmap</a>
</p>

## Alpinoxi Client

Alpinoxi Client is an independent Minecraft launcher maintained by AlpinoxiGD. It is written in Rust and built from the open-source OneLauncher/OneClient codebase under the GPL-3.0-only licence. The launcher is designed to be responsive and modest in memory use; Minecraft itself continues to run in its own Java process with memory selected per instance.

### Accounts and skins

The current codebase supports Microsoft accounts, Ely.by and offline player profiles. Microsoft sign-in must use the official Microsoft/Minecraft authentication flow. Ely.by is configured with the `alpinoxi-client` client ID. Offline profiles are only for servers that deliberately permit offline-mode accounts; they do not bypass Microsoft authentication on online-mode servers.

Account tokens and passwords must never be committed, printed in logs or stored in documentation. See the project rules in `AGENTS.md` when contributing to authentication or skin code.

### In development

The first public beta has not been released. The planned Alpinoxi experience includes a polished skin wardrobe, HUD and keybind settings, social/chat features, curated mod and resource-pack downloads, and separately published Alpinoxi performance work. These are roadmap items, not claims about the current build; see [the roadmap](docs/ALPINOXI_ROADMAP.md) for scope and safeguards.

## Build from source

Prerequisites:

- Rust **1.97+**
- Python 3
- JDK **17+** with `javac`

```sh
cargo +stable build --locked -p oneclient_app --bin oneforall_app
```

Run the focused checks relevant to a change:

```sh
cargo +stable test --locked -p oneclient_auth
cargo +stable test --locked -p oneclient_common -p oneclient_core
cargo +stable fmt --all -- --check
cargo +stable clippy --locked -p oneclient_auth -p oneclient_core -p oneclient_app --all-targets
python3 -B -m unittest discover -s distribution -p 'test_*.py'
```

Packaging and platform details remain under `distribution/`. Do not publish an installer before its exact commit has been built, scanned and manually tested.

## Open source and attribution

Alpinoxi Client is GPL-3.0-only. This repository keeps the upstream notices and third-party credits required by the source and its dependencies—read [ATTRIBUTION.md](ATTRIBUTION.md) before redistributing a build.

Alpinoxi does not rebrand or republish third-party performance mods as its own work. Compatible upstream mods may be offered as clearly attributed dependencies when their licences permit it. Alpinoxi-authored performance modules will be released with source, licence, benchmarks and compatibility notes.

Alpinoxi Client is not affiliated with, endorsed by or verified by Polyfrost, Microsoft, Mojang or Ely.by.

## Türkçe

Alpinoxi Client, AlpinoxiGD tarafından geliştirilen, Minecraft Java Edition için hafif ve açık kaynaklı bir Rust başlatıcısıdır. Kod tabanı GPL-3.0-only lisanslı açık kaynak OneLauncher/OneClient projesinden türetilmiştir; gerekli kaynak ve üçüncü taraf atıfları [ATTRIBUTION.md](ATTRIBUTION.md) içinde korunur.

Microsoft, Ely.by ve çevrimdışı oyuncu profilleri desteklenir. Microsoft hesabı yalnızca resmî Microsoft/Minecraft doğrulama akışıyla kullanılır; Ely.by istemci kimliği `alpinoxi-client` olarak ayarlanır. Çevrimdışı profil, sadece offline-mode kabul eden sunucular içindir ve premium/online-mode doğrulamasını aşmaz.

İlk beta henüz yayımlanmadı. Şık skin arayüzü, HUD ve tuş ayarları, arkadaş/sohbet özellikleri, mod-kaynak paketi kataloğu ve açık kaynak Alpinoxi performans modları planlanmaktadır. Bunlar henüz mevcut sürüm özelliği değildir; kapsam için [yol haritasına](docs/ALPINOXI_ROADMAP.md) bakın.
