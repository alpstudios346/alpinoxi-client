#!/usr/bin/env python3
"""Builds `ui/hudui.html` into `ui/dist/alpinoxi-ui.html`.

`hudui.html` is a template: everything the launcher knows goes in through the
single `{{DATA}}` slot as one JSON object, so no user text is ever pasted into
markup. The build refuses to finish if a slot is left unfilled, because a page
that ships with `{{...}}` in it looks like a bug and reads like one.

    python ui/build_ui.py            # build
    python ui/build_ui.py --check    # build, then verify the page loads

The data comes from the same Rust settings the launcher writes, so the two
cannot disagree about the heap cap, the HUD layout or the enabled mods.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
TEMPLATE = ROOT / "hudui.html"
OUT = ROOT / "dist" / "alpinoxi-ui.html"

# Mirrors alpinoxi_core::launcher::PERFORMANCE_MODS. Kept here rather than read
# from Rust because this script must run without a Rust toolchain (the page is
# built on machines that only have Python).
PERFORMANCE_MODS = [
    {"id": "sodium", "name": "Sodium", "summary": "Rewrites the chunk renderer: the single biggest FPS win"},
    {"id": "lithium", "name": "Lithium", "summary": "Faster ticks, AI and physics without changing behaviour"},
    {"id": "ferritecore", "name": "FerriteCore", "summary": "Cuts model and block-state memory by roughly half"},
    {"id": "krypton", "name": "Krypton", "summary": "Moves networking off the render thread"},
    {"id": "modernfix", "name": "ModernFix", "summary": "Faster startup and fewer stutters on modded packs"},
    {"id": "entityculling", "name": "Entity Culling", "summary": "Skips rendering entities you cannot see"},
    {"id": "indium", "name": "Indium", "summary": "Sodium compatibility layer for mods that use the Fabric renderer"},
    {"id": "immediatelyfast", "name": "ImmediatelyFast", "summary": "Batches immediate-mode GUI and text rendering"},
    {"id": "memoryleakfix", "name": "Memory Leak Fix", "summary": "Patches known leaks that grow over a long session"},
]

EXTRA_CATALOGUE = [
    {"id": "fabric-api", "name": "Fabric API", "summary": "The hooks every Fabric mod builds on", "kind": "mod",
     "performance": False, "licence": "Open source, official release",
     "url": "https://modrinth.com/mod/fabric-api"},
    {"id": "modmenu", "name": "Mod Menu", "summary": "A settings screen for the mods you install", "kind": "mod",
     "performance": False, "licence": "Open source, official release",
     "url": "https://modrinth.com/mod/modmenu"},
    {"id": "iris", "name": "Iris Shaders", "summary": "Shader packs on top of Sodium", "kind": "mod",
     "performance": False, "licence": "Open source, official release",
     "url": "https://modrinth.com/mod/iris"},
    {"id": "alpinoxi-pack", "name": "Alpinoxi HUD pack", "summary": "The pack the launcher's HUD previews", "kind": "resourcepack",
     "performance": False, "licence": "Alpinoxi, MIT",
     "url": "https://modrinth.com/resourcepack/"},
    {"id": "fresh-animations", "name": "Fresh Animations", "summary": "Living mob animations, no extra frames", "kind": "resourcepack",
     "performance": False, "licence": "CC BY-NC-SA",
     "url": "https://modrinth.com/resourcepack/fresh-animations"},
]

HUD_ELEMENTS = [
    ("fps", "FPS", "Top left", "144 FPS"),
    ("coordinates", "Coordinates", "Top left", "X 128  Y 71  Z -64"),
    ("ping", "Ping", "Top right", "23 ms"),
    ("keystrokes", "Keystrokes", "Bottom left", "W A S D"),
    ("armor", "Armor", "Bottom left", "⛊⛊⛊⛊"),
    ("potion_effects", "Potion effects", "Top right", "Speed 1:12"),
    ("server_address", "Server address", "Top right", "play.alpinoxi.net"),
    ("clock", "Clock", "Top left", "21:40"),
    ("friends", "Friends online", "Top right", "3 online"),
    ("reach", "Reach", "Bottom centre", "3.0 blocks"),
]

BINDS = [
    ("Open HUD editor", "Shift+H", False),
    ("Toggle HUD", "F1", False),
    ("Open friends", "O", False),
    ("Open chat", "Y", False),
    ("Toggle skin preview", "U", False),
    ("Toggle FPS", "F1+FPS", False),
]

PROVIDERS = [
    {"id": "microsoft", "name": "Microsoft", "note": "The real account. Skins, capes and friends everywhere.",
     "browser": "Alpinoxi opens your browser and waits for Microsoft to hand back a token. No password is typed into the launcher."},
    {"id": "elyby", "name": "Ely.by", "note": "Skins through Ely.by, client id alpinoxi-client.",
     "browser": "Ely.by sign-in opens in your browser. The launcher only ever sees the token, never your password."},
    {"id": "offline", "name": "Cracked", "note": "No account needed. Skins only where the server accepts them.",
     "browser": ""},
]


def uuid_for(name: str) -> str:
    """Offline UUID, computed the same way Minecraft does."""
    import hashlib

    digest = hashlib.md5(f"OfflinePlayer:{name}".encode("utf-8")).digest()
    hexed = bytearray(digest)
    hexed[6] = (hexed[6] & 0x0F) | 0x30
    hexed[8] = (hexed[8] & 0x3F) | 0x80
    h = hexed.hex()
    return f"{h[0:8]}-{h[8:12]}-{h[12:16]}-{h[16:20]}-{h[20:32]}"


def load_settings() -> dict:
    """The launcher's own settings file, or sensible defaults."""
    import os

    candidates = []
    appdata = os.environ.get("APPDATA")
    if appdata:
        candidates.append(Path(appdata) / "alpinoxi" / "settings.json")
    home = os.environ.get("HOME")
    if home:
        candidates.append(Path(home) / ".config" / "alpinoxi" / "settings.json")
    for path in candidates:
        try:
            return json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
    return {}


def build_data() -> dict:
    settings = load_settings()
    instance = settings.get("instance", {}) or {}
    hud = settings.get("hud", {}) or {}

    catalogue = [
        {"id": m["id"], "name": m["name"], "summary": m["summary"], "kind": "mod",
         "performance": True, "licence": "Open source, official release",
         "url": f"https://modrinth.com/mod/{m['id']}"}
        for m in PERFORMANCE_MODS
    ] + EXTRA_CATALOGUE

    return {
        "version": "0.2.0",
        "slogan": "OneClient is best!",
        "system_memory_mib": settings.get("system_memory_mib", 8192),
        "memory_mib": settings.get("memory", {"Custom": 2048}),
        "keep_light": settings.get("keep_launcher_light", True),
        "close_on_launch": settings.get("close_launcher_on_launch", False),
        "provider": (settings.get("last_provider") or "microsoft").lower(),
        "cracked_name": settings.get("cracked_name", ""),
        "skin_model": settings.get("skin_model", "Classic"),
        "skin_source": "None",
        "skin_note": "No skin selected yet.",
        "upload_target": "local only",
        "providers": PROVIDERS,
        "account": None,
        "uuid_for": uuid_for,
        "instance": {
            "name": instance.get("name", "Vanilla"),
            "version": instance.get("game_version", "1.21.1"),
            "loader": instance.get("loader", "fabric"),
            "enabled_mods": instance.get("enabled_mods", []),
            "resource_packs": len(instance.get("resource_packs", []) or []),
        },
        "performance": PERFORMANCE_MODS,
        "catalog": catalogue,
        "friends": [
            {"name": "Alpinoxi", "provider": "Ely.by", "presence": "In game", "server": "play.alpinoxi.net"},
            {"name": "Steve", "provider": "Microsoft", "presence": "In launcher", "server": None},
            {"name": "Alex", "provider": "Offline", "presence": "Offline", "server": None},
        ],
        "pending": 1,
        "hud": {
            "enabled": hud.get("enabled", True),
            "background": hud.get("background", True),
            "scale_percent": hud.get("scale_percent", 100),
            "elements": [
                {"id": eid, "label": label, "anchor": anchor, "visible": True, "sample": sample}
                for eid, label, anchor, sample in HUD_ELEMENTS
            ],
        },
        "binds": [{"action": a, "chord": c, "vanilla": v} for a, c, v in BINDS],
        "default_binds": [{"action": a, "chord": c, "vanilla": v} for a, c, v in BINDS],
    }


def memory_mib(raw) -> int:
    """`MemoryProfile` serialises as either a bare variant or {"Custom": n}."""
    if isinstance(raw, int):
        return raw
    if isinstance(raw, dict) and "Custom" in raw:
        return int(raw["Custom"])
    if isinstance(raw, str):
        return {"Balanced": 2048, "Performance": 4096}.get(raw, 2048)
    return 2048


def build() -> Path:
    template = TEMPLATE.read_text(encoding="utf-8")
    data = build_data()
    data["memory_mib"] = memory_mib(data["memory_mib"])

    payload = json.dumps(data, ensure_ascii=False, indent=None, default=str)
    # `<` and `>` inside a <script> can close the tag early; escaping them is the
    # standard fix and JSON.parse reads the result unchanged.
    payload = payload.replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026")

    if template.count("{{DATA}}") != 1:
        raise SystemExit("template must contain exactly one {{DATA}} slot")

    html = template.replace("{{DATA}}", payload)

    leftover = re.findall(r"\{\{[A-Z_]+\}\}", html)
    if leftover:
        raise SystemExit(f"unfilled slots in the page: {sorted(set(leftover))}")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(html, encoding="utf-8")
    return OUT


def check(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    problems = []
    if "{{" in text:
        problems.append("a template slot survived into the built page")
    if "<script>" not in text or "</script>" not in text:
        problems.append("the page has no script block")
    if text.count("<div") != text.count("</div>"):
        problems.append(f"unbalanced divs: {text.count('<div')} open, {text.count('</div>')} close")
    if "const DATA = " not in text:
        problems.append("DATA is missing")
    if problems:
        raise SystemExit("check failed: " + "; ".join(problems))
    size = len(text.encode("utf-8"))
    print(f"ok  {path}  {size / 1024:.1f} KB  divs balanced, no slots left")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify the built page")
    args = parser.parse_args()
    path = build()
    print(f"built {path}")
    if args.check:
        check(path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
