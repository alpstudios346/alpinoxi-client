// A headless pass over every screen and control, so "it is buggy" becomes a
// list of concrete failures. Runs the page against a stub DOM, clicks what can
// be clicked, and reports what threw.
const fs = require('fs');

const html = fs.readFileSync('ui/index.html', 'utf8');
const script = html.split('<script>').pop().split('</script>')[0];

function makeEl(id) {
  const el = {
    id, hidden: false, value: '', textContent: '', innerHTML: '',
    style: {}, dataset: {}, children: [],
    classList: { add() {}, remove() {}, toggle() {}, contains() { return false; } },
    setAttribute() {}, getAttribute() { return null; }, removeAttribute() {},
    appendChild(c) { this.children.push(c); return c; },
    querySelector() { return makeEl(id + '-q'); },
    querySelectorAll() { return []; },
    addEventListener() {}, removeEventListener() {},
    scrollTop: 0, scrollHeight: 0, onclick: null, oninput: null, onchange: null, onkeydown: null
  };
  return el;
}

const registry = {};
const document = {
  getElementById: (id) => (registry[id] ||= makeEl(id)),
  querySelector: (s) => (registry['sel:' + s] ||= makeEl(s)),
  querySelectorAll: () => [],
  createElement: (tag) => makeEl(tag + ':' + Math.random().toString(36).slice(2)),
  addEventListener() {}
};

const errors = [];
global.document = document;
global.window = { addEventListener() {}, alpinoxiIpc: null, open() {} };
global.setTimeout = () => 0;
global.clearTimeout = () => {};
global.TextEncoder = TextEncoder;

try {
  new Function('document', 'window', script)(document, global.window);
} catch (error) {
  errors.push(`script threw on load: ${error.message}`);
}

// Every control the page wires up should have a handler once the script ran.
const expected = [
  'cracked-go', 'browser-go', 'sign-out', 'top-launch', 'home-launch',
  'ram-slider', 'skin-apply', 'friend-add', 'chat-send', 'mod-search',
  'mod-perf', 'import-scan', 'bind-reset', 'set-hud', 'set-hudbg',
  'hud-scale', 'inst-name', 'inst-version', 'inst-loader'
];
for (const id of expected) {
  const el = registry[id];
  if (!el) { errors.push(`${id}: never touched by the page`); continue; }
  const wired = el.onclick || el.oninput || el.onchange || el.onkeydown;
  if (!wired) errors.push(`${id}: no handler`);
}

// Clicking the controls must not throw.
for (const id of ['cracked-go', 'browser-go', 'sign-out', 'bind-reset', 'mod-perf', 'import-scan']) {
  const el = registry[id];
  if (el && el.onclick) {
    try { el.onclick({ preventDefault() {}, target: { value: '' } }); }
    catch (error) { errors.push(`${id} threw: ${error.message}`); }
  }
}

if (errors.length) {
  console.log(`FAIL (${errors.length})`);
  errors.forEach((e) => console.log('  - ' + e));
  process.exit(1);
}
console.log(`OK: ${expected.length} controls wired, no handler threw`);
