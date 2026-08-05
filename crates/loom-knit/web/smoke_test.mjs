// Headless correctness check for the wasm build: runs WebGame under Node
// with mock CanvasRenderingContext2d/KeyboardEvent objects (no browser
// needed) and verifies it constructs, renders, and responds to input
// without throwing. Doesn't check pixel output -- that still needs a real
// browser -- but it does exercise the actual compiled .wasm end-to-end
// (board generation's solvability retry loop, getRandomValues via
// wasm_js/crypto, cursor movement, pick-up), not just "it compiles."
//
// Run after regenerating pkg/ (see README.md in this directory):
//   node smoke_test.mjs

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import init, { WebGame } from './pkg/knitui.js';

const here = dirname(fileURLToPath(import.meta.url));
const wasmBytes = readFileSync(join(here, 'pkg', 'knitui_bg.wasm'));

function makeMockCtx() {
  const draws = [];
  return {
    draws,
    fillStyle: '',
    font: '',
    globalAlpha: 1,
    textBaseline: '',
    fillRect(x, y, w, h) { draws.push({ op: 'rect', x, y, w, h }); },
    fillText(text, x, y) { draws.push({ op: 'text', text, x, y }); },
    measureText(text) { return { width: text.length * 9.6 }; },
  };
}

function key(k) {
  return { key: k, ctrlKey: false, shiftKey: false, altKey: false };
}

async function main() {
  await init({ module_or_path: wasmBytes });

  const ctx = makeMockCtx();
  const game = new WebGame(ctx, 100, 40, 16);

  game.render();
  const firstFrameDraws = ctx.draws.length;
  if (firstFrameDraws === 0) {
    throw new Error('render() produced zero draw calls -- board generation or rendering is broken');
  }

  game.handle_key(key('ArrowRight'));
  game.handle_key(key('ArrowDown'));
  game.render();

  game.handle_key(key('Enter'));
  game.render();

  console.log(`OK: constructed, rendered ${ctx.draws.length} total draw calls across 3 frames, no exceptions.`);
}

main().catch((err) => {
  console.error('SMOKE TEST FAILED:', err);
  process.exit(1);
});
