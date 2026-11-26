// Runs the page's script against a stub DOM and reports what threw.
const fs = require('fs');
const data = JSON.parse(fs.readFileSync('check_data.json', 'utf8'));
const els = {};
function el(id) {
  if (!els[id]) els[id] = {
    id, hidden: false, value: '', textContent: '', innerHTML: '',
    style: {}, dataset: {}, classList: { add(){}, remove(){}, toggle(){}, contains(){return false} },
    setAttribute(){}, getAttribute(){return null}, appendChild(c){ (this.children ||= []).push(c); },
    querySelector(){ return el(id + '-child'); }, querySelectorAll(){ return []; },
    addEventListener(){}, scrollTop: 0, scrollHeight: 0
  };
  return els[id];
}
global.document = {
  getElementById: el,
  querySelector: (s) => el(s),
  querySelectorAll: () => [],
  createElement: (t) => { const e = el(t + Math.random()); e.tagName = t; return e; },
  addEventListener(){}
};
global.window = { addEventListener(){}, alpinoxiIpc: null };
global.setTimeout = () => 0;
global.clearTimeout = () => {};
const src = fs.readFileSync('page_script.js', 'utf8').replace('const DATA = __DATA__;', 'const DATA = ' + JSON.stringify(data) + ';');
try {
  new Function(src)();
  console.log('OK: the page script runs');
} catch (error) {
  console.log('FAIL: ' + error.message);
  console.log(error.stack.split('\n').slice(0, 4).join('\n'));
  process.exit(1);
}
