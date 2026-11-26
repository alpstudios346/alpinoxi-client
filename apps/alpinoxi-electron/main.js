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
  })
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
