
"use strict";

// ---------- data the launcher hands over ----------
const DATA = {"version": "0.2.0", "slogan": "OneClient is best!", "system_memory_mib": 6082, "memory_mib": 2048, "keep_light": true, "close_on_launch": false, "provider": "microsoft", "cracked_name": "", "skin_model": "Classic", "skin_source": "None", "skin_note": "No skin selected yet.", "upload_target": "local only", "providers": [{"id": "microsoft", "name": "Microsoft", "note": "The real account. Skins, capes and friends everywhere.", "browser": "Alpinoxi opens your browser and waits for Microsoft to hand back a token. No password is typed into the launcher."}, {"id": "elyby", "name": "Ely.by", "note": "Skins through Ely.by, client id alpinoxi-client.", "browser": "Ely.by sign-in opens in your browser. The launcher only ever sees the token, never your password."}, {"id": "offline", "name": "Cracked", "note": "No account needed. Skins only where the server accepts them.", "browser": ""}], "account": null, "uuid_for": "\u003cfunction uuid_for at 0x000001E5F03336A0\u003e", "instance": {"name": "Vanilla", "version": "1.21.1", "loader": "fabric", "enabled_mods": [], "resource_packs": 0}, "performance": [{"id": "sodium", "name": "Sodium", "summary": "Rewrites the chunk renderer: the single biggest FPS win"}, {"id": "lithium", "name": "Lithium", "summary": "Faster ticks, AI and physics without changing behaviour"}, {"id": "ferritecore", "name": "FerriteCore", "summary": "Cuts model and block-state memory by roughly half"}, {"id": "krypton", "name": "Krypton", "summary": "Moves networking off the render thread"}, {"id": "modernfix", "name": "ModernFix", "summary": "Faster startup and fewer stutters on modded packs"}, {"id": "entityculling", "name": "Entity Culling", "summary": "Skips rendering entities you cannot see"}, {"id": "indium", "name": "Indium", "summary": "Sodium compatibility layer for mods that use the Fabric renderer"}, {"id": "immediatelyfast", "name": "ImmediatelyFast", "summary": "Batches immediate-mode GUI and text rendering"}, {"id": "memoryleakfix", "name": "Memory Leak Fix", "summary": "Patches known leaks that grow over a long session"}], "catalog": [{"id": "sodium", "name": "Sodium", "summary": "Rewrites the chunk renderer: the single biggest FPS win", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/sodium"}, {"id": "lithium", "name": "Lithium", "summary": "Faster ticks, AI and physics without changing behaviour", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/lithium"}, {"id": "ferritecore", "name": "FerriteCore", "summary": "Cuts model and block-state memory by roughly half", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/ferritecore"}, {"id": "krypton", "name": "Krypton", "summary": "Moves networking off the render thread", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/krypton"}, {"id": "modernfix", "name": "ModernFix", "summary": "Faster startup and fewer stutters on modded packs", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/modernfix"}, {"id": "entityculling", "name": "Entity Culling", "summary": "Skips rendering entities you cannot see", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/entityculling"}, {"id": "indium", "name": "Indium", "summary": "Sodium compatibility layer for mods that use the Fabric renderer", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/indium"}, {"id": "immediatelyfast", "name": "ImmediatelyFast", "summary": "Batches immediate-mode GUI and text rendering", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/immediatelyfast"}, {"id": "memoryleakfix", "name": "Memory Leak Fix", "summary": "Patches known leaks that grow over a long session", "kind": "mod", "performance": true, "licence": "Open source, official release", "url": "https://modrinth.com/mod/memoryleakfix"}, {"id": "fabric-api", "name": "Fabric API", "summary": "The hooks every Fabric mod builds on", "kind": "mod", "performance": false, "licence": "Open source, official release", "url": "https://modrinth.com/mod/fabric-api"}, {"id": "modmenu", "name": "Mod Menu", "summary": "A settings screen for the mods you install", "kind": "mod", "performance": false, "licence": "Open source, official release", "url": "https://modrinth.com/mod/modmenu"}, {"id": "iris", "name": "Iris Shaders", "summary": "Shader packs on top of Sodium", "kind": "mod", "performance": false, "licence": "Open source, official release", "url": "https://modrinth.com/mod/iris"}, {"id": "alpinoxi-pack", "name": "Alpinoxi HUD pack", "summary": "The pack the launcher's HUD previews", "kind": "resourcepack", "performance": false, "licence": "Alpinoxi, MIT", "url": "https://modrinth.com/resourcepack/"}, {"id": "fresh-animations", "name": "Fresh Animations", "summary": "Living mob animations, no extra frames", "kind": "resourcepack", "performance": false, "licence": "CC BY-NC-SA", "url": "https://modrinth.com/resourcepack/fresh-animations"}], "friends": [{"name": "Alpinoxi", "provider": "Ely.by", "presence": "In game", "server": "play.alpinoxi.net"}, {"name": "Steve", "provider": "Microsoft", "presence": "In launcher", "server": null}, {"name": "Alex", "provider": "Offline", "presence": "Offline", "server": null}], "pending": 1, "hud": {"enabled": true, "background": true, "scale_percent": 100, "elements": [{"id": "fps", "label": "FPS", "anchor": "Top left", "visible": true, "sample": "144 FPS"}, {"id": "coordinates", "label": "Coordinates", "anchor": "Top left", "visible": true, "sample": "X 128  Y 71  Z -64"}, {"id": "ping", "label": "Ping", "anchor": "Top right", "visible": true, "sample": "23 ms"}, {"id": "keystrokes", "label": "Keystrokes", "anchor": "Bottom left", "visible": true, "sample": "W A S D"}, {"id": "armor", "label": "Armor", "anchor": "Bottom left", "visible": true, "sample": "⛊⛊⛊⛊"}, {"id": "potion_effects", "label": "Potion effects", "anchor": "Top right", "visible": true, "sample": "Speed 1:12"}, {"id": "server_address", "label": "Server address", "anchor": "Top right", "visible": true, "sample": "play.alpinoxi.net"}, {"id": "clock", "label": "Clock", "anchor": "Top left", "visible": true, "sample": "21:40"}, {"id": "friends", "label": "Friends online", "anchor": "Top right", "visible": true, "sample": "3 online"}, {"id": "reach", "label": "Reach", "anchor": "Bottom centre", "visible": true, "sample": "3.0 blocks"}]}, "binds": [{"action": "Open HUD editor", "chord": "Shift+H", "vanilla": false}, {"action": "Toggle HUD", "chord": "F1", "vanilla": false}, {"action": "Open friends", "chord": "O", "vanilla": false}, {"action": "Open chat", "chord": "Y", "vanilla": false}, {"action": "Toggle skin preview", "chord": "U", "vanilla": false}, {"action": "Toggle FPS", "chord": "F1+FPS", "vanilla": false}], "default_binds": [{"action": "Open HUD editor", "chord": "Shift+H", "vanilla": false}, {"action": "Toggle HUD", "chord": "F1", "vanilla": false}, {"action": "Open friends", "chord": "O", "vanilla": false}, {"action": "Open chat", "chord": "Y", "vanilla": false}, {"action": "Toggle skin preview", "chord": "U", "vanilla": false}, {"action": "Toggle FPS", "chord": "F1+FPS", "vanilla": false}]};

const SCREENS = ["home", "instances", "skin", "friends", "mods", "import", "settings"];
const LABELS = {
  home: "Home", instances: "Instances", skin: "Skin", friends: "Friends",
  mods: "Mods & packs", import: "Import", settings: "Settings"
};
const ANCHORS = ["Top left", "Top right", "Bottom left", "Bottom right", "Bottom centre"];

const state = {
  screen: "home",
  data: DATA,
  signedIn: false,
  arming: null,
  selectedElement: "fps",
  chat: [{ who: "System", text: "Welcome to Alpinoxi chat. Press Y in game to open it.", local: false }],
  chatChannel: "global",
  modQuery: "",
  modKind: null,
  modPerf: false,
  heap: null,
  toastTimer: null
};

const $ = (id) => document.getElementById(id);
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({
  "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"
}[c]));

function send(command) {
  if (!window.alpinoxiIpc) return Promise.resolve(null);
  return window.alpinoxiIpc.send(command).catch(() => null);
}

function toast(text) {
  const el = $("toast");
  el.textContent = text;
  el.classList.add("show");
  clearTimeout(state.toastTimer);
  state.toastTimer = setTimeout(() => el.classList.remove("show"), 2600);
}

function pillClass(provider) {
  return provider === "Microsoft" ? "ms" : provider === "Ely.by" ? "ely" : "crack";
}

function planHeap() {
  // The launcher's answer wins when there is one: it owns the cap.
  if (state.heap) return state.heap;
  const requested = state.data.memory_mib;
  const ceiling = Math.min(Math.floor(state.data.system_memory_mib / 2), 8192);
  return Math.max(512, Math.min(requested, ceiling));
}

// ---------- login ----------
function renderProviders() {
  const host = $("providers");
  host.innerHTML = "";
  state.data.providers.forEach((p) => {
    const b = document.createElement("button");
    b.className = "provider" + (state.data.provider === p.id ? " sel" : "");
    b.innerHTML = `<div class="name">${esc(p.name)}</div><div class="note">${esc(p.note)}</div>`;
    b.onclick = () => {
      state.data.provider = p.id;
      $("cracked-box").style.display = p.id === "offline" ? "block" : "none";
      $("browser-box").style.display = p.id === "offline" ? "none" : "block";
      $("browser-note").textContent = p.browser;
      $("login-notice").textContent = p.note;
      renderProviders();
    };
    host.appendChild(b);
  });
}

function signIn(profile) {
  state.data.account = profile;
  state.signedIn = true;
  $("login-view").hidden = true;
  $("app-view").hidden = false;
  renderAll();
  toast(`Signed in as ${profile.name}`);
}

$("cracked-go").onclick = () => {
  const name = $("cracked-name").value.trim();
  if (name.length < 3) {
    $("login-notice").textContent = "A name needs at least three characters.";
    return;
  }
  signIn({ name, provider: "Offline", uuid: state.data.uuid_for(name) });
};

$("browser-go").onclick = () => {
  $("login-notice").textContent =
    "Alpinoxi opens your browser. Nothing is typed into this window: the code goes straight to the provider.";
};

// ---------- shell ----------
function renderNav() {
  const nav = $("nav");
  nav.innerHTML = "";
  SCREENS.forEach((s) => {
    const b = document.createElement("button");
    b.innerHTML = `<span class="dot"></span><span>${LABELS[s]}</span>`;
    if (s === state.screen) b.setAttribute("aria-current", "page");
    b.onclick = () => go(s);
    nav.appendChild(b);
  });
}

function go(screen) {
  state.screen = screen;
  SCREENS.forEach((s) => { $("view-" + s).hidden = s !== screen; });
  renderNav();
  document.querySelector(".screen").scrollTop = 0;
}

document.querySelectorAll("[data-goto]").forEach((b) => {
  b.onclick = () => go(b.dataset.goto);
});

$("sign-out").onclick = () => {
  state.signedIn = false;
  state.data.account = null;
  $("app-view").hidden = true;
  $("login-view").hidden = false;
};

$("top-launch").onclick = $("home-launch").onclick = () => {
  toast(`Starting ${state.data.instance.name} · ${state.data.instance.version} with ${planHeap()} MiB`);
};

// ---------- home ----------
function renderHome() {
  const inst = state.data.instance;
  $("home-instance").textContent = inst.name;
  $("home-instance-meta").textContent =
    `${inst.version} · ${inst.loader} · ${inst.enabled_mods.length} mods · ${inst.resource_packs} resource packs`;
  $("home-pills").innerHTML = [
    `<span class="pill accent">${planHeap()} MiB heap</span>`,
    `<span class="pill mint">G1GC tuned</span>`,
    `<span class="pill dim">${state.data.keep_light ? "Launcher stays light" : "Launcher kept open"}</span>`
  ].join("");
  $("home-ram-note").textContent = planHeap() < state.data.memory_mib
    ? `${state.data.memory_mib} MiB was requested, but this machine has ${state.data.system_memory_mib} MiB of RAM, so Alpinoxi will use ${planHeap()} MiB.`
    : "";

  const acct = state.data.account || { name: "-", provider: "Offline", uuid: "-" };
  $("acct-name").textContent = acct.name;
  $("acct-prov").textContent = acct.provider;
  $("acct-prov").className = "pill " + pillClass(acct.provider);
  $("acct-uuid").textContent = "UUID " + acct.uuid;
  $("acct-skin").textContent = state.data.skin_note;
  $("acct-slogan").textContent = state.data.slogan;

  $("top-slogan").textContent = state.data.slogan;
  $("top-meta").textContent =
    `${inst.version} · ${planHeap()} MiB heap · ${inst.enabled_mods.length} mods enabled`;
  $("account-chip").querySelector(".who").textContent = acct.name;
  $("account-chip").querySelector(".prov").textContent = acct.provider;
  $("account-chip").querySelector(".prov").className = "prov";
  $("account-chip").querySelector(".dot").style.background =
    acct.provider === "Microsoft" ? "var(--ms)" : acct.provider === "Ely.by" ? "var(--ely)" : "var(--crack)";

  document.querySelectorAll("[data-mem]").forEach((b) => {
    b.classList.toggle("sel", Number(b.dataset.mem) === state.data.memory_mib);
  });
  $("ram-slider").value = state.data.memory_mib;
  $("ram-label").textContent =
    `${planHeap()} MiB heap · system RAM ${state.data.system_memory_mib} MiB`;

  $("home-perf").innerHTML = state.data.performance.map((m) => {
    const on = state.data.instance.enabled_mods.includes(m.id);
    return `<button class="btn ${on ? "sel" : ""}" data-mod="${esc(m.id)}" title="${esc(m.summary)}">${esc(m.name)}</button>`;
  }).join(" ");
  $("home-perf").querySelectorAll("[data-mod]").forEach((b) => {
    b.onclick = () => toggleMod(b.dataset.mod);
  });
}

document.querySelectorAll("[data-mem]").forEach((b) => {
  b.onclick = () => { state.data.memory_mib = Number(b.dataset.mem); renderHome(); };
});
$("ram-slider").oninput = (e) => {
  state.data.memory_mib = Number(e.target.value);
  renderHome();
};
$("ram-slider").onchange = async (e) => {
  const reply = await send({ cmd: "set_memory", mib: Number(e.target.value) });
  if (reply && reply.heap) state.heap = reply.heap;
  renderHome();
};

async function toggleMod(id) {
  const list = state.data.instance.enabled_mods;
  const at = list.indexOf(id);
  if (at === -1) list.push(id); else list.splice(at, 1);
  renderHome();
  renderMods();
  toast(at === -1 ? "Added to the instance" : "Removed from the instance");
  await send({ cmd: "toggle_mod", id });
}

// ---------- instances ----------
function renderInstances() {
  const inst = state.data.instance;
  $("inst-name").value = inst.name;
  $("inst-version").value = inst.version;
  $("inst-loader").value = inst.loader;
  $("inst-mods").textContent = inst.enabled_mods.length
    ? inst.enabled_mods.join(", ")
    : "No mods yet.";
}
$("inst-name").oninput = (e) => { state.data.instance.name = e.target.value; renderHome(); };
$("inst-version").oninput = (e) => { state.data.instance.version = e.target.value; renderHome(); };
$("inst-loader").onchange = (e) => { state.data.instance.loader = e.target.value; renderHome(); };

// ---------- skin ----------
function renderSkin() {
  const fig = $("skin-figure");
  fig.className = "skin-figure" + (state.data.skin_model === "Slim" ? " slim" : "");
  $("skin-pills").innerHTML = [
    `<span class="pill accent">${esc(state.data.skin_model)}</span>`,
    `<span class="pill dim">${esc(state.data.skin_source)}</span>`
  ].join("");
  $("skin-visibility").textContent = state.data.skin_note;
  $("skin-upload").textContent = state.data.upload_target;
  document.querySelectorAll("[data-model]").forEach((b) => {
    b.classList.toggle("sel", b.dataset.model === state.data.skin_model.toLowerCase());
  });
}
document.querySelectorAll("[data-model]").forEach((b) => {
  b.onclick = () => {
    state.data.skin_model = b.dataset.model === "slim" ? "Slim" : "Classic";
    renderSkin();
  };
});
$("skin-apply").onclick = () => {
  const path = $("skin-path").value.trim();
  if (!path) { toast("Type the path to a PNG first"); return; }
  state.data.skin_note = "Local file accepted. Only servers that accept custom skins will show it.";
  state.data.skin_source = "Local";
  renderSkin();
  toast("Local skin applied");
};

// ---------- friends ----------
function renderFriends() {
  const host = $("friend-list");
  host.innerHTML = state.data.friends.length ? "" : `<div class="empty"><strong>Nobody yet</strong>Add a name above.</div>`;
  state.data.friends.forEach((f) => {
    const colour = f.presence === "In game" ? "var(--mint)" : f.presence === "In launcher" ? "var(--warn)" : "#2a4268";
    const row = document.createElement("div");
    row.className = "friend";
    row.innerHTML = `
      <span class="dot" style="background:${colour}"></span>
      <span class="name">${esc(f.name)}</span>
      <span class="pill ${pillClass(f.provider)}">${esc(f.provider)}</span>
      <span class="where">${esc(f.presence)}${f.server ? " · " + esc(f.server) : ""}</span>`;
    host.appendChild(row);
  });
  $("friend-requests").innerHTML = state.data.pending
    ? `<div class="rule"></div><div class="eyebrow">Requests</div><div class="sub" style="color:var(--warn)">${state.data.pending} pending</div>`
    : "";
  renderChat();
}

function renderChat() {
  $("chat-tabs").innerHTML = ["global", "party"].map((c) =>
    `<button class="btn ${state.chatChannel === c ? "sel" : ""}" data-chat="${c}">${c[0].toUpperCase() + c.slice(1)}</button>`
  ).join("");
  $("chat-tabs").querySelectorAll("[data-chat]").forEach((b) => {
    b.onclick = () => { state.chatChannel = b.dataset.chat; renderChat(); };
  });
  const log = $("chatlog");
  log.innerHTML = state.chat.map((m) =>
    `<p><span class="who">${esc(m.who)}</span> ${esc(m.text)}</p>`
  ).join("");
  log.scrollTop = log.scrollHeight;
}

$("friend-add").onclick = () => {
  const name = $("friend-name").value.trim();
  if (!name) return;
  if (state.data.friends.some((f) => f.name.toLowerCase() === name.toLowerCase())) {
    toast("Already in your list");
    return;
  }
  state.data.friends.push({ name, provider: "Offline", presence: "Offline", server: null });
  state.data.pending += 1;
  $("friend-name").value = "";
  renderFriends();
  toast(`Request sent to ${name}`);
};

$("chat-send").onclick = () => {
  const text = $("chat-input").value.trim();
  if (!text) return;
  state.chat.push({ who: (state.data.account || {}).name || "You", text, local: true });
  $("chat-input").value = "";
  renderChat();
};
$("chat-input").onkeydown = (e) => { if (e.key === "Enter") $("chat-send").click(); };

// ---------- catalogue ----------
function renderMods() {
  const q = state.modQuery.toLowerCase();
  const host = $("mod-list");
  const rows = state.data.catalog.filter((m) =>
    (!state.modKind || m.kind === state.modKind) &&
    (!state.modPerf || m.performance) &&
    (!q || m.name.toLowerCase().includes(q) || m.summary.toLowerCase().includes(q))
  );
  if (!rows.length) {
    host.innerHTML = `<div class="empty"><strong>Nothing matches</strong>Try a shorter search or clear the filters.</div>`;
    return;
  }
  host.innerHTML = rows.map((m) => {
    const on = state.data.instance.enabled_mods.includes(m.id);
    return `<div class="mod">
      <div class="body">
        <div class="nm">${esc(m.name)}</div>
        <div class="sum">${esc(m.summary)}</div>
        <span class="pill dim">${esc(m.kind === "mod" ? "Mod" : "Resource pack")}</span>
        ${m.performance ? `<span class="pill mint">Performance</span>` : ""}
        <span class="pill accent">${esc(m.licence)}</span>
      </div>
      <div class="row">
        <button class="btn" data-open="${esc(m.url)}">Page</button>
        <button class="btn ${on ? "sel" : ""}" data-mod="${esc(m.id)}" style="min-width:132px">${on ? "Remove" : "Add to instance"}</button>
      </div>
    </div>`;
  }).join("");
  host.querySelectorAll("[data-mod]").forEach((b) => { b.onclick = () => toggleMod(b.dataset.mod); });
  host.querySelectorAll("[data-open]").forEach((b) => {
    b.onclick = () => {
      const url = b.dataset.open;
      if (window.alpinoxiIpc) {
        send({ cmd: "open_external", url });
        toast("Opening the project page in your browser");
      } else {
        window.open(url, "_blank", "noopener");
      }
    };
  });
  document.querySelectorAll("[data-kind]").forEach((b) => {
    b.classList.toggle("sel", b.dataset.kind === state.modKind);
  });
  $("mod-perf").classList.toggle("sel", state.modPerf);
}
$("mod-search").oninput = (e) => { state.modQuery = e.target.value; renderMods(); };
document.querySelectorAll("[data-kind]").forEach((b) => {
  b.onclick = () => {
    state.modKind = state.modKind === b.dataset.kind ? null : b.dataset.kind;
    renderMods();
  };
});
$("mod-perf").onclick = () => { state.modPerf = !state.modPerf; renderMods(); };

// ---------- import ----------
$("import-scan").onclick = () => {
  const path = $("import-path").value.trim();
  if (!path) { $("import-result").textContent = "Type the folder your other launcher uses."; return; }
  $("import-result").textContent =
    "Alpinoxi reads that folder, lists what it can copy (worlds, resource packs, configs) and skips accounts and libraries. Nothing is moved.";
  toast("Scan needs the Rust side: the launcher does this natively");
};

// ---------- settings ----------
function renderSettings() {
  $("set-light").setAttribute("aria-pressed", String(state.data.keep_light));
  $("set-close").setAttribute("aria-pressed", String(state.data.close_on_launch));
  $("set-hud").setAttribute("aria-pressed", String(state.data.hud.enabled));
  $("set-hudbg").setAttribute("aria-pressed", String(state.data.hud.background));
  $("hud-scale").value = state.data.hud.scale_percent;
  $("hud-scale-label").textContent = `${state.data.hud.scale_percent}%`;
  renderBinds();
  renderHudEditor();
}

function bindSwitch(id, key) {
  $(id).onclick = () => {
    state.data[key] = !state.data[key];
    renderSettings();
    renderHome();
  };
}
bindSwitch("set-light", "keep_light");
bindSwitch("set-close", "close_on_launch");

$("set-hud").onclick = () => {
  state.data.hud.enabled = !state.data.hud.enabled;
  renderSettings();
};
$("set-hudbg").onclick = () => {
  state.data.hud.background = !state.data.hud.background;
  renderSettings();
  renderHudEditor();
};
$("hud-scale").oninput = (e) => {
  state.data.hud.scale_percent = Number(e.target.value);
  $("hud-scale-label").textContent = `${e.target.value}%`;
};

function renderBinds() {
  const host = $("bind-list");
  host.innerHTML = state.data.binds.map((b, i) => `
    <div class="bind">
      <span class="act">${esc(b.action)}</span>
      ${b.vanilla ? `<span class="pill warn">vanilla key</span>` : ""}
      <button class="key ${state.arming === i ? "arming" : ""}" data-bind="${i}">
        ${state.arming === i ? "press a key…" : esc(b.chord)}
      </button>
    </div>`).join("");
  host.querySelectorAll("[data-bind]").forEach((btn) => {
    btn.onclick = () => {
      state.arming = Number(btn.dataset.bind);
      renderBinds();
      toast("Press the new combination, Escape to cancel");
    };
  });
  $("bind-conflict").textContent = "";
}
$("bind-reset").onclick = () => {
  state.data.binds = state.data.default_binds.slice();
  renderBinds();
  toast("Keybinds reset");
};

window.addEventListener("keydown", (e) => {
  if (state.arming === null) return;
  e.preventDefault();
  if (e.key === "Escape") { state.arming = null; renderBinds(); return; }
  if (["Shift", "Control", "Alt", "Meta"].includes(e.key)) return;
  const parts = [];
  if (e.shiftKey) parts.push("Shift");
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
  const chord = parts.join("+");
  const taken = state.data.binds.find((b, i) => i !== state.arming && b.chord === chord);
  if (taken) {
    $("bind-conflict").textContent = `${taken.action} already uses ${chord}, pick another key.`;
    return;
  }
  state.data.binds[state.arming].chord = chord;
  state.arming = null;
  renderBinds();
  toast(`Bound to ${chord}`);
});

function renderHudEditor() {
  const stage = $("hud-stage");
  stage.innerHTML = `<div class="horizon"></div>`;
  const groups = {};
  state.data.hud.elements.forEach((el) => {
    (groups[el.anchor] = groups[el.anchor] || []).push(el);
  });
  Object.entries(groups).forEach(([anchor, els]) => {
    const wrap = document.createElement("div");
    const pos = {
      "Top left": { left: "12px", top: "12px", align: "left" },
      "Top right": { right: "12px", top: "12px", align: "right" },
      "Bottom left": { left: "12px", bottom: "12px", align: "left" },
      "Bottom right": { right: "12px", bottom: "12px", align: "right" },
      "Bottom centre": { left: "50%", bottom: "12px", transform: "translateX(-50%)", align: "center" }
    }[anchor] || { left: "12px", top: "12px" };
    Object.assign(wrap.style, pos, { position: "absolute", display: "flex", flexDirection: "column", gap: "4px" });
    els.forEach((el) => {
      const chip = document.createElement("div");
      chip.className = "hud-el" + (el.visible ? "" : " off");
      chip.textContent = el.sample || el.label;
      chip.style.position = "static";
      wrap.appendChild(chip);
    });
    stage.appendChild(wrap);
  });

  const host = $("hud-elements");
  host.innerHTML = "";
  state.data.hud.elements.forEach((el) => {
    const row = document.createElement("div");
    row.className = "el-row";
    row.innerHTML = `
      <span class="lbl">${esc(el.label)}</span>
      <span class="pill dim">${esc(el.anchor)}</span>
      <button class="switch" aria-pressed="${el.visible}" aria-label="${esc(el.label)}"></button>`;
    row.querySelector(".switch").onclick = () => {
      el.visible = !el.visible;
      state.selectedElement = el.id;
      renderHudEditor();
      send({ cmd: "toggle_hud_element", id: el.id });
    };
    row.onclick = (e) => {
      if (e.target.classList.contains("switch")) return;
      state.selectedElement = el.id;
      renderHudEditor();
    };
    if (el.id === state.selectedElement) row.style.background = "rgba(79,209,255,.06)";
    host.appendChild(row);
  });

  const group = $("anchor-group");
  group.innerHTML = "";
  ANCHORS.forEach((a) => {
    const b = document.createElement("button");
    b.textContent = a;
    const el = state.data.hud.elements.find((e) => e.id === state.selectedElement);
    if (el && el.anchor === a) b.className = "sel";
    b.onclick = () => {
      if (el) el.anchor = a;
      renderHudEditor();
    };
    group.appendChild(b);
  });
}

// ---------- boot ----------
function renderAll() {
  renderNav();
  renderHome();
  renderInstances();
  renderSkin();
  renderFriends();
  renderMods();
  renderSettings();
  $("rail-version").textContent = `v${state.data.version} · Rust native`;
  // Electron hands the page its settings and answers commands; the webview
// build has neither, so both paths are guarded here.
async function boot() {
  if (!window.alpinoxiIpc) return;
  try {
    const info = await window.alpinoxiIpc.boot();
    if (info && info.settings) {
      state.data.memory_mib = info.settings.memory_mib;
      state.data.system_memory_mib = info.system_memory_mib;
      state.data.instance.enabled_mods = info.settings.instance.enabled_mods || [];
      state.data.hud.enabled = info.settings.hud.enabled;
      state.data.hud.background = info.settings.hud.background;
      state.data.hud.scale_percent = info.settings.hud.scale_percent;
      if (info.settings.hud.elements) {
        state.data.hud.elements.forEach((el) => {
          if (el.id in info.settings.hud.elements) el.visible = info.settings.hud.elements[el.id];
        });
      }
      if (info.heap_mib) state.heap = info.heap_mib;
      if (info.version) state.data.version = info.version;
    }
    renderAll();
  } catch (error) {
    toast('Could not reach the launcher: ' + error.message);
  }
}
boot();

$("login-slogan").textContent = state.data.slogan;
}

$("login-slogan").textContent = state.data.slogan;
$("cracked-name").value = state.data.cracked_name || "";
renderProviders();
$("cracked-box").style.display = state.data.provider === "offline" ? "block" : "none";
$("browser-box").style.display = state.data.provider === "offline" ? "none" : "block";
$("browser-note").textContent = state.data.providers.find((p) => p.id === state.data.provider).browser;
if (state.data.account) signIn(state.data.account);
