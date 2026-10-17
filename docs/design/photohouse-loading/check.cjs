// Executar com: node docs/design/photohouse-loading/check.cjs
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const html = fs.readFileSync(path.join(__dirname, 'index.html'), 'utf8');
const script = html.match(/<script>([\s\S]*?)<\/script>/)[1];
assert(fs.existsSync(path.join(__dirname, 'assets/housinho-frames.webp')));
assert(html.includes('data-variant="Preto"'));
assert(html.includes('data-variant="Off-white"'));
function setup(reduced) {
  const pending = new Map();
  let requestId = 0;
  const makeNode = () => ({
    textContent: '', clientWidth: 280, disabled: true, dataset: {}, attrs: {},
    style: {setProperty(k, v) {this[k] = v;}},
    getBoundingClientRect: () => ({width: 30}),
    setAttribute(k, v) {this.attrs[k] = v;},
    addEventListener(k, fn) {this[k] = fn;},
    getContext: () => ({clearRect() {}, drawImage() {}})
  });
  const views = ['Preto', 'Off-white'].map(name => {
    const nodes = {};
    return {nodes, dataset: {variant: name}, querySelector(s) {return nodes[s] ??= makeNode();}};
  });
  const replay = makeNode(), announcement = makeNode();
  const root = {
    querySelectorAll: () => views,
    querySelector: s => s === '.ph-replay' ? replay : s === '.ph-announcement' ? announcement : null
  };
  let image;
  const context = {
    document: {getElementById: () => root, createElement: () => ({getContext: () => ({measureText: () => ({width: 168})})})},
    Image: class {constructor() {image = this; this.naturalWidth = 3920; this.naturalHeight = 3962;}},
    ResizeObserver: class {observe() {}},
    matchMedia: () => ({matches: reduced, addEventListener() {}}),
    performance: {now: () => 0},
    requestAnimationFrame: fn => {pending.set(++requestId, fn); return requestId;},
    cancelAnimationFrame: id => pending.delete(id)
  };
  vm.runInNewContext(script, context);
  return {views, replay, announcement, image, pending, tick(now) {const callbacks = [...pending.values()]; pending.clear(); callbacks.forEach(fn => fn(now));}};
}
const normal = setup(false);
assert.equal(normal.pending.size, 0, 'não iniciar antes de carregar os quadros');
normal.image.onload();
const expect = (state, percentage, frame) => state.views.forEach(view => {
  assert.equal(view.nodes['.ph-value'].textContent, percentage);
  assert.equal(view.nodes['.ph-art'].dataset.frame, String(frame));
  assert.equal(view.nodes['.ph-track'].attrs['aria-valuenow'], String(Number(percentage)));
});
expect(normal, '000', 0);
normal.tick(2080);
expect(normal, '050', 25);
normal.tick(4159);
expect(normal, '099', 50);
normal.tick(4160);
expect(normal, '100', 51);
assert.equal(normal.pending.size, 0, 'nenhum loop após 100%');
normal.tick(10000);
expect(normal, '100', 51);
normal.replay.click();
expect(normal, '000', 0);
assert.equal(normal.pending.size, 1, 'replay reinicia uma única linha de tempo');
const reduced = setup(true);
reduced.image.onload();
expect(reduced, '100', 51);
assert.equal(reduced.pending.size, 0);
const failed = setup(false);
failed.image.onerror();
assert.equal(failed.replay.disabled, true);
assert(failed.announcement.textContent.includes('Não foi possível'));
console.log('PASS: ambas as versões; 0%, 50%, 99%, 100%; quadro final congelado; replay; movimento reduzido; falha de asset.');
