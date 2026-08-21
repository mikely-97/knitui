// Headless correctness check for the wasm build -- see
// ../../loom-knit/web/smoke_test.mjs for the full rationale. Now exercises
// the full Shell<PicrossGame> state machine (main menu -> Campaign ->
// track select -> level intro -> play -> help -> quit-to-menu), not a
// bare gameplay loop. picross's main_menu_items() is [Campaign, Quit]
// only. Run after regenerating pkg/ (see README.md):
//   node smoke_test.mjs

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import init, { WebGame } from './pkg/pictui.js';

const here = dirname(fileURLToPath(import.meta.url));
const wasmBytes = readFileSync(join(here, 'pkg', 'pictui_bg.wasm'));

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

  game.render(); // main menu (Campaign, Quit)
  const menuDraws = ctx.draws.length;
  if (menuDraws === 0) {
    throw new Error('render() produced zero draw calls on the main menu -- chrome rendering is broken');
  }
  if (game.should_quit()) {
    throw new Error('should_quit() must start false');
  }

  // Main menu -> Enter selects Campaign (index 0) -> CampaignSelect.
  game.handle_key(key('Enter'));
  game.render();

  // Track 0 -> CampaignLevelIntro (no blessing screen -- picross has none).
  game.handle_key(key('Enter'));
  game.render();

  // Start the level -> Playing.
  game.handle_key(key('Enter'));
  game.render();
  if (ctx.draws.length <= menuDraws) {
    throw new Error('render() after starting a campaign level produced no new draw calls -- puzzle rendering is broken');
  }

  game.handle_key(key('ArrowRight'));
  game.handle_key(key('ArrowDown'));
  game.tick();
  game.render();

  game.handle_key(key('Enter')); // fill a cell
  game.render();

  game.handle_key(key('x')); // cross a cell
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

  console.log(`OK: constructed, navigated menu -> campaign -> play -> help -> menu, rendered ${ctx.draws.length} total draw calls, no exceptions.`);
}

main().catch((err) => {
  console.error('SMOKE TEST FAILED:', err);
  process.exit(1);
});
