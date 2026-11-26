// Alpinoxi Client: the Electron window.
//
// The interface is one HTML file (`ui/index.html`), the same file the Rust
// shell shows, so the two windows cannot drift apart. What differs is the
// renderer: this one is Chromium, bundled, so it draws the same on every
// machine and can use the newer CSS and animation features Lunar-style
// interfaces lean on.
//
// The main process owns the state. The page is a view: it sends a command and
// gets JSON back, and every decision (the heap cap, the mod toggles, the
// keybind conflicts) happens here, where it can be tested.

const { app, BrowserWindow, ipcMain, shell, nativeTheme } = require('electron');
const path = require('node:path');
const fs = require('node:fs');
const os = require('node:os');

const VERSION = '0.3.0';
const SLOGAN = 'OneClient is best!';

// Half of system RAM, and never more than 8 GiB: the same rule the Rust side
// enforces, because a launcher that hands the game all of RAM is how a world
// turns into a crash.
const HEAP_CEILING_MIB = 8192;

function dataDir() {
  const base = process.env.APPDATA || path.join(os.homedir(), '.config');
  return path.join(base, 'alpinoxi');
}

const settingsPath = () => path.join(dataDir(), 'settings.json');
const bridgePath = () => path.join(dataDir(), 'bridge.json');

function systemMemoryMib() {
  // `os.totalmem()` is bytes; the launcher thinks in mebibytes everywhere.
  return Math.floor(os.totalmem() / (1024 * 1024));
}

function defaultSettings() {
  return {
    system_memory_mib: systemMemoryMib(),
    memory_mib: 2048,
    instance: {
      name: 'Vanilla',
      version: '1.21.1',
      loader: 'fabric',
      enabled_mods: [],
      resource_packs: []
    },
    hud: {
      enabled: true,
      background: true,
      scale_percent: 100
    },
    keep_launcher_light: true,
    close_launcher_on_launch: false,
    last_provider: 'microsoft',
    cracked_name: '',
    skin_model: 'Classic'
  };
}

function loadSettings() {
  try {
    return { ...defaultSettings(), ...JSON.parse(fs.readFileSync(settingsPath(), 'utf8')) };
  } catch {
    return defaultSettings();
  }
}

function saveSettings(settings) {
  fs.mkdirSync(dataDir(), { recursive: true });
  fs.writeFileSync(settingsPath(), JSON.stringify(settings, null, 2));
  // The in-game mods read this, so the HUD preview and the HUD in game cannot
  // disagree about layout or scale.
  fs.writeFileSync(
    bridgePath(),
    JSON.stringify(
      {
        version: 1,
        hud: settings.hud,
        heap_mib: planHeap(settings),
        performance_mods: settings.instance.enabled_mods
      },
      null,
      2
    )
  );
}

// Where each provider's sign-in page lives. `alpinoxi-client` is the Ely.by
// client id the Rust side uses too.
const AUTH_URLS = {
  microsoft: 'https://login.live.com/oauth20_authorize.srf?client_id=00000000402b5328&response_type=code&scope=service%3A%3Auser.auth.xboxlive.com%3A%3AMBI_SSL&redirect_uri=https%3A%2F%2Flogin.live.com%2Foauth20_desktop.srf',
  elyby: 'https://account.ely.by/oauth2/v1?client_id=alpinoxi-client&response_type=code&scope=account_info%20minecraft_server_session'
};

// Minecraft's offline UUID: MD5 of "OfflinePlayer:<name>", version 3.
function offlineUuid(name) {
  const crypto = require('node:crypto');
  const digest = crypto.createHash('md5').update(`OfflinePlayer:${name}`, 'utf8').digest();
  digest[6] = (digest[6] & 0x0f) | 0x30;
  digest[8] = (digest[8] & 0x3f) | 0x80;
  const hex = digest.toString('hex');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function planHeap(settings) {
  const cap = Math.min(Math.floor(settings.system_memory_mib / 2), HEAP_CEILING_MIB);
  return Math.max(512, Math.min(Number(settings.memory_mib) || 2048, cap));
}

// ---------------------------------------------------------------------------
// commands
// ---------------------------------------------------------------------------

const handlers = {
  heap: (s) => ({ heap: planHeap(s) }),
  set_memory: (s, cmd) => {
    s.memory_mib = Math.max(512, Math.min(Number(cmd.mib) || 2048, 16384));
    saveSettings(s);
    return { heap: planHeap(s) };
  },
  toggle_mod: (s, cmd) => {
    const list = s.instance.enabled_mods;
    const at = list.indexOf(cmd.id);
    if (at === -1) list.push(cmd.id);
    else list.splice(at, 1);
    saveSettings(s);
    return { enabled: at === -1, mods: list };
  },
  toggle_hud_element: (s, cmd) => {
    s.hud.elements = s.hud.elements || {};
    s.hud.elements[cmd.id] = !s.hud.elements[cmd.id];
    saveSettings(s);
    return { visible: !s.hud.elements[cmd.id] };
  },
  set_hud: (s, cmd) => {
    Object.assign(s.hud, cmd.value || {});
    saveSettings(s);
    return { hud: s.hud };
  },
  save: (s) => {
    saveSettings(s);
    return { ok: true };
  },
  set_cracked_name: (s, cmd) => {
    s.cracked_name = String(cmd.name || '').slice(0, 16);
    saveSettings(s);
    return { cracked_name: s.cracked_name };
  },
  open_external: (s, cmd) => {
    // Only https, so a page cannot ask the launcher to run a local file.
    if (typeof cmd.url === 'string' && cmd.url.startsWith('https://')) shell.openExternal(cmd.url);
    return { ok: true };
  },
  info: (s) => ({
    version: VERSION,
    slogan: SLOGAN,
    system_memory_mib: s.system_memory_mib,
    heap_mib: planHeap(s),
    data_dir: dataDir()
  }),
  // Signing in is the launcher's job, not the page's: the page sends a
  // provider and a name, and the launcher answers with an account.
  sign_in: (s, cmd) => {
    if (cmd.provider === 'offline') {
      const name = String(cmd.name || '').trim();
      if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) {
        return { error: 'a name is 3 to 16 letters, numbers or underscores' };
      }
      s.cracked_name = name;
      saveSettings(s);
      return { account: { name, provider: 'Offline', uuid: offlineUuid(name) } };
    }
    // Microsoft and Ely.by open the provider's own page. The token comes back
    // to a loopback listener, never through this window.
    const url = AUTH_URLS[cmd.provider];
    if (!url) return { error: `unknown provider: ${cmd.provider}` };
    shell.openExternal(url);
    return { pending: true, opened: url };
  }
};

let state = null;

function handle(body) {
  let cmd;
  try {
    cmd = typeof body === 'string' ? JSON.parse(body) : body;
  } catch (error) {
    return { error: `bad command: ${error.message}` };
  }
  const handler = handlers[cmd && cmd.cmd];
  if (!handler) return { error: `unknown command: ${cmd && cmd.cmd}` };
  try {
    return handler(state, cmd);
  } catch (error) {
    return { error: error.message };
  }
}

// ---------------------------------------------------------------------------
// window
// ---------------------------------------------------------------------------

function createWindow() {
  const win = new BrowserWindow({
    width: 1180,
    height: 760,
    minWidth: 960,
    minHeight: 620,
    backgroundColor: '#070e1e',
    show: false,
    autoHideMenuBar: true,
    title: 'Alpinoxi Client',
    icon: path.join(__dirname, 'build', 'icon.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      spellcheck: false
    }
  });

  win.once('ready-to-show', () => win.show());

  const log = (line) => {
    try {
      fs.appendFileSync(path.join(dataDir(), 'launcher.log'), `${new Date().toISOString()} ${line}\n`);
    } catch {
      /* a launcher that cannot log still has to run */
    }
  };

  win.webContents.on('render-process-gone', (_event, details) => {
    log(`renderer gone: ${details.reason} (exit ${details.exitCode})`);
    // One reload is a hiccup; a second failure is a real bug, so say so.
    if (!win.__reloaded) {
      win.__reloaded = true;
      win.reload();
    } else {
      win.loadFile(path.join(__dirname, 'ui', 'index.html'));
      win.webContents.once('did-finish-load', () =>
        win.webContents.send('alpinoxi:notice', 'The window restarted after a crash.')
      );
    }
  });

  win.webContents.on('did-fail-load', (_event, code, description, url) => {
    log(`load failed: ${code} ${description} ${url}`);
  });

  // A page that throws leaves a blank window, which reads as "nothing
  // happened". Catch it and say what it was.
  win.webContents.on('console-message', (event) => {
    const level = event.level ?? arguments[1];
    if (level === 'error' || level === 3) {
      log(`page error: ${event.message ?? ''} ${event.lineNumber ?? ''}`);
    }
  });

  process.on('uncaughtException', (error) => log(`uncaught: ${error.stack || error.message}`));

  win.loadFile(path.join(__dirname, 'ui', 'index.html'));

  // A link never navigates the launcher window; it opens in the browser.
  win.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith('https://')) shell.openExternal(url);
    return { action: 'deny' };
  });
  win.webContents.on('will-navigate', (event, url) => {
    if (!url.startsWith('file://')) {
      event.preventDefault();
      if (url.startsWith('https://')) shell.openExternal(url);
    }
  });

  return win;
}

ipcMain.handle('alpinoxi:command', (_event, body) => handle(body));
ipcMain.handle('alpinoxi:boot', () => ({
  ...handlers.info(state),
  settings: state
}));

app.whenReady().then(() => {
  nativeTheme.themeSource = 'dark';
  state = loadSettings();
  saveSettings(state);
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (state) saveSettings(state);
  app.quit();
});

// Tests run this file with `--test-command`: no window, just the logic.
if (process.argv.includes('--self-test')) {
  state = defaultSettings();
  state.system_memory_mib = 8192;
  state.memory_mib = 16000;
  console.log(JSON.stringify({ heap: planHeap(state), info: handlers.info(state) }));
  app.exit(0);
}
