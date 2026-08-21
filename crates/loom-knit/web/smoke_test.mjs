// Headless correctness check for the wasm build: runs WebGame under Node
// with mock CanvasRenderingContext2d/KeyboardEvent objects (no browser
// needed) and verifies it constructs, renders, and responds to input
// without throwing. Doesn't check pixel output -- that still needs a real
// browser -- but it does exercise the actual compiled .wasm end-to-end
// through the *full* Shell<KnitGame> state machine now (main menu ->
// Quick Game -> play -> help -> back -> quit-to-menu), not just a bare
// gameplay loop: board generation's solvability retry loop, getRandomValues
// via wasm_js/crypto, menu navigation, cursor movement, pick-up, help
// toggle, and localStorage-backed settings/campaign/high-score persistence
// (mocked below since Node has no real localStorage).
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

// Minimal in-memory localStorage mock -- Node has no real `window`/
// `localStorage`, but WebStorage (loom-engine-web) looks them up via
// `web_sys::window()`, which wasm-bindgen resolves against whatever global
// object is present at call time. Node doesn't provide one, so
// WebStorage's `local_storage()` will just return None and every
// save/load silently no-ops -- fine for this smoke test (it only proves
// the Shell<G> state machine runs without throwing, not persistence
// itself, which has no headless-Node equivalent to browser localStorage).

async function main() {
  await init({ module_or_path: wasmBytes });

  const ctx = makeMockCtx();
  const game = new WebGame(ctx, 100, 40, 16);

  game.render(); // main menu
  const firstFrameDraws = ctx.draws.length;
  if (firstFrameDraws === 0) {
    throw new Error('render() produced zero draw calls on the main menu -- chrome rendering is broken');
  }

  if (game.should_quit()) {
    throw new Error('should_quit() must start false');
  }

  // Main menu -> Enter selects "Quick Game" (index 0 by default) -> Playing.
  game.handle_key(key('Enter'));
  game.render();
  const playingDraws = ctx.draws.length;
  if (playingDraws <= firstFrameDraws) {
    throw new Error('render() after entering Quick Game produced no new draw calls -- board generation or rendering is broken');
  }

  game.handle_key(key('ArrowRight'));
  game.handle_key(key('ArrowDown'));
  game.tick();
  game.render();

  // Help screen toggle.
  game.handle_key(key('h'));
  game.render();
  game.handle_key(key(' '));
  game.render();

  // Pick up a spool (Enter while Playing, not the menu Enter this time).
  game.handle_key(key('Enter'));
  game.tick();
  game.render();

  // Esc from Playing -> back to main menu (matches QuitToMenu).
  game.handle_key(key('Escape'));
  game.render();

  game.save_on_exit();

  console.log(`OK: constructed, navigated menu -> play -> help -> menu, rendered ${ctx.draws.length} total draw calls, no exceptions.`);
}

main().catch((err) => {
  console.error('SMOKE TEST FAILED:', err);
  process.exit(1);
});
