// Runs the launcher's own main.js logic in isolation: the command layer, the
// log writer, and the boot reply. A page bug and a launcher bug should never
// look the same again.
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');

const home = fs.mkdtempSync(path.join(os.tmpdir(), 'alpinoxi-test-'));
process.env.APPDATA = home;

const source = fs.readFileSync('main.js', 'utf8');

// Grab one top-level function (or const) by name, brace-matched.
function grab(name) {
  const start = source.search(new RegExp(`^(async function|function|const|let) ${name}\\b`, 'm'));
  if (start < 0) throw new Error(`${name} is not in main.js`);
  const open = source.indexOf('{', start);
  const semi = source.indexOf(';', start);
  if (semi > 0 && open > semi) return source.slice(start, semi + 1);
  let depth = 0;
  for (let i = open; i < source.length; i++) {
    if (source[i] === '{') depth++;
    else if (source[i] === '}') {
      depth--;
      if (depth === 0) {
        // a trailing `;` belongs to the expression, not the block
        return source.slice(start, source[i + 1] === ';' ? i + 2 : i + 1);
      }
    }
  }
  throw new Error(`${name} has no end`);
}

const wanted = ['dataDir', 'settingsPath', 'bridgePath', 'systemMemoryMib', 'defaultSettings',
  'loadSettings', 'saveSettings', 'planHeap', 'offlineUuid', 'log'];
const constants = source.match(/^const [A-Z_]+ = [^;]+;/gm) || [];
const table = {};
const fn = new Function('require', 'module', 'exports', 'log', 'fs', 'path', 'os', 'process',
  `${constants.join('\n')}
   ${wanted.map(grab).join('\n')}
   module.exports.dataDir = dataDir;
   module.exports.systemMemoryMib = systemMemoryMib;
   module.exports.defaultSettings = defaultSettings;
   module.exports.loadSettings = loadSettings;
   module.exports.saveSettings = saveSettings;
   module.exports.planHeap = planHeap;
   module.exports.offlineUuid = offlineUuid;
   module.exports.log = log;`);
fn(require, { exports: table }, table, null, fs, path, os, process);

const api = table;

// 1. The launcher's log must write where it says it does.
api.log('first line');
api.log('second line');
const logPath = path.join(home, 'alpinoxi', 'launcher.log');
assert.ok(fs.existsSync(logPath), `launcher.log missing at ${logPath}`);
const lines = fs.readFileSync(logPath, 'utf8').trim().split('\n');
assert.equal(lines.length, 2, 'two lines are two lines');

// 2. The folder is %APPDATA%/alpinoxi, and the settings round-trip.
assert.equal(api.dataDir(), path.join(home, 'alpinoxi'));
const settings = api.defaultSettings();
settings.memory_mib = 3072;
settings.last_provider = 'elyby';
api.saveSettings(settings);
const back = api.loadSettings();
assert.equal(back.memory_mib, 3072);
assert.equal(back.last_provider, 'elyby');

// 3. The heap never exceeds half the machine, and never exceeds the ceiling.
assert.equal(api.planHeap({ memory_mib: 99999, system_memory_mib: 8192 }), 4096);
assert.equal(api.planHeap({ memory_mib: 99999, system_memory_mib: 65536 }), 8192);
assert.equal(api.planHeap({ memory_mib: 1024, system_memory_mib: 8192 }), 1024);

// 4. The offline UUID is Minecraft's, for names everyone checks.
assert.equal(api.offlineUuid('Notch'), 'b50ad385-829d-3141-a216-7e7d7539ba7f');
assert.equal(api.offlineUuid('Alpinoxi'), '7cce3372-aec0-3d25-9588-23579d5fcd8a');

console.log('OK: log writer, settings round-trip, heap cap, offline UUID');
