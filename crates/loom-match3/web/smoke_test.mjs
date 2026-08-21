// Headless correctness check for the wasm build -- see
// ../../loom-knit/web/smoke_test.mjs for the full rationale. Now exercises
// the full Shell<M3Game> state machine (main menu -> Quick Game -> play ->
// help -> quit-to-menu), not a bare gameplay loop. Run after regenerating
// pkg/ (see README.md in this directory):
//   node smoke_test.mjs

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import init, { WebGame } from './pkg/m3tui.js';

const here = dirname(fileURLToPath(import.meta.url));
const wasmBytes = readFileSync(join(here, 'pkg', 'm3tui_bg.wasm'));

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

  // Main menu -> Enter selects "Quick Game" (index 0) -> Playing.
  game.handle_key(key('Enter'));
  game.render();
  if (ctx.draws.length <= menuDraws) {
    throw new Error('render() after entering Quick Game produced no new draw calls -- board generation or rendering is broken');
  }

  game.handle_key(key('ArrowRight'));
  game.handle_key(key('Enter'));
  game.handle_key(key('ArrowDown'));
  game.handle_key(key('Enter'));

  // Several ticks to exercise the match/cascade phase pipeline.
  for (let i = 0; i < 10; i++) {
    game.tick();
    game.render();
  }

  // Help screen toggle.
  game.handle_key(key('h'));
  game.render();
  game.handle_key(key(' '));
  game.render();

  // Esc from Playing -> back to main menu.
  game.handle_key(key('Escape'));
  game.render();

  game.save_on_exit();

  console.log(`OK: constructed, navigated menu -> play -> help -> menu, rendered ${ctx.draws.length} total draw calls, no exceptions.`);
}

main().catch((err) => {
  console.error('SMOKE TEST FAILED:', err);
  process.exit(1);
});
