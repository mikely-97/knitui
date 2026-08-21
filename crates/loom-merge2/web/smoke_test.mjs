// Headless correctness check for the wasm build -- see
// ../../loom-knit/web/smoke_test.mjs for the full rationale. Now exercises
// the full Shell<M2Game> state machine (main menu -> Custom Game editor ->
// play -> inventory -> help -> quit-to-menu), not a bare gameplay loop.
// Note merge2's main menu has no Quick Game item (main_menu_items() is
// [Custom Game, Campaign, Endless, Options, Quit]), so the first Enter
// opens the Custom Game editor, and a second Enter is needed to actually
// start playing. Run after regenerating pkg/ (see README.md):
//   node smoke_test.mjs

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import init, { WebGame } from './pkg/m2tui.js';

const here = dirname(fileURLToPath(import.meta.url));
const wasmBytes = readFileSync(join(here, 'pkg', 'm2tui_bg.wasm'));

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

  game.render(); // main menu
  const menuDraws = ctx.draws.length;
  if (menuDraws === 0) {
    throw new Error('render() produced zero draw calls on the main menu -- chrome rendering is broken');
  }
  if (game.should_quit()) {
    throw new Error('should_quit() must start false');
  }

  // Main menu -> Enter opens Custom Game editor (index 0, no Quick Game).
  game.handle_key(key('Enter'));
  game.render();
  const editorDraws = ctx.draws.length;
  if (editorDraws <= menuDraws) {
    throw new Error('render() after opening Custom Game produced no new draw calls');
  }

  // Enter again on the (unedited) editor -> start playing.
  game.handle_key(key('Enter'));
  game.render();
  if (ctx.draws.length <= editorDraws) {
    throw new Error('render() after starting Custom Game produced no new draw calls -- board generation or rendering is broken');
  }

  game.handle_key(key('ArrowRight'));
  game.handle_key(key('Enter'));
  for (let i = 0; i < 5; i++) {
    game.tick();
    game.render();
  }

  // Inventory mode round trip.
  game.handle_key(key('i'));
  game.render();
  game.handle_key(key('Escape'));
  game.render();

  // Help screen toggle.
  game.handle_key(key('h'));
  game.render();
  game.handle_key(key(' '));
  game.render();

  // Esc from Playing -> back to main menu.
  game.handle_key(key('Escape'));
  game.render();

  game.save_on_exit();

  console.log(`OK: constructed, navigated menu -> custom game -> play -> inventory -> help -> menu, rendered ${ctx.draws.length} total draw calls, no exceptions.`);
}

main().catch((err) => {
  console.error('SMOKE TEST FAILED:', err);
  process.exit(1);
});
