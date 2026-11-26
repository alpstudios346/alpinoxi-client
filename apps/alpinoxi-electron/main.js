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
    skin_model: 'Classic',
    account: null
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

// One entry per provider Alpinoxi can sign in with. A desktop binary cannot
// keep a client secret, so both flows use PKCE and a loopback redirect, which
// is what OAuth 2.0 prescribes for native apps.
const AUTH = {
  microsoft: {
    label: 'Microsoft',
    clientId: '00000000402b5328', // Minecraft's own public launcher id
    authorize: 'https://login.live.com/oauth20_authorize.srf',
    token: 'https://login.live.com/oauth20_token.srf',
    scope: 'XboxLive.signin offline_access',
    xbox: 'https://user.auth.xboxlive.com/user/authenticate',
    xsts: 'https://xsts.auth.xboxlive.com/xsts/authorize',
    minecraft: 'https://api.minecraftservices.com/authentication/login_with_xbox',
    profile: 'https://api.minecraftservices.com/minecraft/profile'
  },
  elyby: {
    label: 'Ely.by',
    clientId: 'alpinoxi-client',
    authorize: 'https://account.ely.by/oauth2/v1',
    token: 'https://account.ely.by/api/oauth2/v1/token',
    scope: 'account_info offline_access minecraft_server_session',
    info: 'https://account.ely.by/api/account/v1/info'
  }
};

// How long a sign-in may sit waiting for the browser.
const SIGN_IN_TIMEOUT_MS = 5 * 60 * 1000;

function base64url(buffer) {
  return buffer.toString('base64').replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

// One sign-in at a time: a second click joins the one already waiting.
let signingIn = null;

/// Starts a provider sign-in: listen on loopback, open the page, wait for the
/// code, exchange it, and hand the account to the window.
async function startSignIn(provider) {
  if (signingIn) {
    notice('A sign-in is already waiting in your browser.');
    return;
  }
  signingIn = signIn(provider);
  try {
    await signingIn;
  } finally {
    signingIn = null;
  }
}

async function signIn(provider) {
  const spec = AUTH[provider];
  if (!spec) throw new Error(`unknown provider: ${provider}`);

  const verifier = base64url(crypto.randomBytes(48));
  const challenge = base64url(crypto.createHash('sha256').update(verifier).digest());
  const state = base64url(crypto.randomBytes(24));

  const { server, redirectUri } = await listenLoopback();
  notice(`Waiting for ${spec.label} to hand back the sign-in…`);

  try {
    const url = new URL(spec.authorize);
    url.searchParams.set('client_id', spec.clientId);
    url.searchParams.set('response_type', 'code');
    url.searchParams.set('redirect_uri', redirectUri);
    url.searchParams.set('scope', spec.scope);
    url.searchParams.set('state', state);
    url.searchParams.set('code_challenge', challenge);
    url.searchParams.set('code_challenge_method', 'S256');
    if (provider === 'elyby') url.searchParams.set('description', 'Alpinoxi Client');

    log(`sign-in: ${spec.label} page opened, waiting for ${redirectUri}`);
    const code = await waitForCode(server, state, spec.label, url.toString());
    log(`sign-in: ${spec.label} returned a code`);
    const tokens = await exchangeCode(provider, code, verifier, redirectUri);
    log(`sign-in: ${spec.label} exchanged the code for a token`);
    const account = await resolveAccount(provider, tokens);
    log(`sign-in: ${spec.label} signed in as ${account.name}`);
    saveAccount(account);
    notice(`${account.name} is signed in.`);
    send({ type: 'signed_in', account });
  } finally {
    server.close();
  }
}

/// Listens on a free port of 127.0.0.1 and returns the redirect it answers on.
function listenLoopback() {
  return new Promise((resolve, reject) => {
    const server = http.createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      resolve({ server, redirectUri: `http://127.0.0.1:${port}/callback` });
    });
  });
}

/// Opens the page and resolves with the authorization code the browser sends
/// back. Anything that is not the callback is answered and ignored.
function waitForCode(server, state, label, authorizeUrl) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      server.close();
      reject(new Error(`${label} did not answer in time`));
    }, SIGN_IN_TIMEOUT_MS);

    const finish = (error, code) => {
      clearTimeout(timer);
      server.removeAllListeners('request');
      if (error) reject(error); else resolve(code);
    };

    server.on('request', (request, response) => {
      const url = new URL(request.url, 'http://127.0.0.1');
      if (url.pathname !== '/callback') {
        response.writeHead(404).end();
        return;
      }
      const sent = url.searchParams.get('state');
      const code = url.searchParams.get('code');
      const error = url.searchParams.get('error');

      const page = (title, body) =>
        `<!doctype html><meta charset="utf-8"><title>${title}</title>` +
        `<body style="font:16px/1.6 system-ui;background:#0b1220;color:#e6eefc;display:grid;place-items:center;height:100vh;margin:0">` +
        `<div style="text-align:center"><h1 style="font-size:20px">${title}</h1><p>${body}</p></div></body>`;

      // `finish` settles the promise this handler is waiting on, so anything
      // it reports must be caught here: an uncaught rejection would take the
      // whole launcher down.
      const settle = (failure, value) => {
        try {
          finish(failure, value);
        } catch (unused) {
          // nothing is waiting any more: the answer page has already been sent
        }
      };

      if (error) {
        response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
        response.end(page('Sign-in cancelled', 'You can close this tab and try again in Alpinoxi.'));
        settle(new Error(error));
        return;
      }
      if (!code || sent !== state) {
        response.writeHead(400, { 'content-type': 'text/html; charset=utf-8' });
        response.end(page('That did not match', 'Close this tab and start again from Alpinoxi.'));
        settle(new Error('the provider answered with an unexpected state'));
        return;
      }
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end(page('Signed in', 'You can close this tab and go back to Alpinoxi.'));
      settle(null, code);
    });

    Promise.resolve(shell.openExternal(authorizeUrl)).catch((error) => {
      clearTimeout(timer);
      server.close();
      reject(new Error(`the browser could not be opened: ${error.message}`));
    });
  });
}

async function exchangeCode(provider, code, verifier, redirectUri) {
  const spec = AUTH[provider];
  const body = new URLSearchParams({
    grant_type: 'authorization_code',
    code,
    redirect_uri: redirectUri,
    client_id: spec.clientId,
    code_verifier: verifier
  });
  const response = await fetch(spec.token, {
    method: 'POST',
    headers: { 'content-type': 'application/x-www-form-urlencoded', accept: 'application/json' },
    body
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(data.error_description || data.error || `${spec.label} refused the code (${response.status})`);
  }
  return data;
}

async function resolveAccount(provider, tokens) {
  if (provider === 'elyby') {
    const response = await fetch(AUTH.elyby.info, {
      headers: { authorization: `Bearer ${tokens.access_token}`, accept: 'application/json' }
    });
    if (!response.ok) throw new Error(`Ely.by refused the token (${response.status})`);
    const info = await response.json();
    return {
      name: info.username,
      uuid: info.uuid,
      provider: 'Ely.by',
      token: tokens.access_token,
      refresh: tokens.refresh_token || null
    };
  }

  // Microsoft: the Microsoft token has to be traded for Xbox, then XSTS, then
  // Minecraft, in that order. Each step fails with its own message.
  const msToken = tokens.access_token;
  const xbox = await postJson(AUTH.microsoft.xbox, {
    Properties: { AuthMethod: 'RPS', SiteName: 'user.auth.xboxlive.com', RpsTicket: `d=${msToken}` },
    RelyingParty: 'http://auth.xboxlive.com',
    TokenType: 'JWT'
  }, 'Xbox Live');

  const xsts = await postJson(AUTH.microsoft.xsts, {
    Properties: { SandboxId: 'RETAIL', UserTokens: [xbox.Token] },
    RelyingParty: 'rp://api.minecraftservices.com/',
    TokenType: 'JWT'
  }, 'XSTS');

  const claim = xsts.DisplayClaims.xui[0];
  const mc = await postJson(AUTH.microsoft.minecraft, {
    identityToken: `XBL3.0 x=${claim.uhs};${xsts.Token}`
  }, 'Minecraft');

  const profileResponse = await fetch(AUTH.microsoft.profile, {
    headers: { authorization: `Bearer ${mc.access_token}`, accept: 'application/json' }
  });
  if (profileResponse.status === 404) throw new Error('that account does not own Minecraft');
  if (!profileResponse.ok) throw new Error(`Minecraft profile failed (${profileResponse.status})`);
  const profile = await profileResponse.json();

  return {
    name: profile.name,
    uuid: profile.id,
    provider: 'Microsoft',
    token: mc.access_token,
    refresh: tokens.refresh_token || null
  };
}

async function postJson(url, body, label) {
  const response = await fetch(url, {
    method: 'POST',
    headers: { 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify(body)
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    const detail = data.XErr ? ` (XErr ${data.XErr}: ${xboxError(data.XErr)})` : '';
    throw new Error(`${label} sign-in failed${detail}`);
  }
  return data;
}

function xboxError(code) {
  return {
    2148916233: 'that Microsoft account has no Xbox profile, make one first',
    2148916235: 'Xbox Live is not available in that country',
    2148916236: 'that account needs an adult to verify it',
    2148916237: 'that account needs an adult to verify it',
    2148916238: 'that account is a child account and needs a family'
  }[code] || 'Microsoft refused the account';
}

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
    // Microsoft and Ely.by sign in through their own page, in the player's
    // browser. The code comes back to a loopback listener on this machine, and
    // only the launcher ever sees the token.
    if (!AUTH[cmd.provider]) return { error: `unknown provider: ${cmd.provider}` };
    startSignIn(cmd.provider).catch((error) => {
      log(`sign-in: failed: ${error.message}`);
      notice(`Sign-in failed: ${error.message}`);
    });
    return { pending: true };
  }
};

let state = null;
let mainWindow = null;

// Tells the window something happened. The page listens for these and shows
// them as a notice, so a sign-in that fails says why instead of doing nothing.
function notice(text) {
  log(text);
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('alpinoxi:notice', text);
  }
}

function send(message) {
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send('alpinoxi:event', message);
  }
}

// The signed-in account is the launcher's, so it is saved beside the settings.
function saveAccount(account) {
  state.account = account;
  saveSettings(state);
}

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
      mainWindow = win;
  win.on('closed', () => {
    if (mainWindow === win) mainWindow = null;
  });

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

  mainWindow = win;
  win.on('closed', () => {
    if (mainWindow === win) mainWindow = null;
  });

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
  account: state.account || null,
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
