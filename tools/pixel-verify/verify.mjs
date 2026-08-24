// Pixel-level browser verification for all 4 loom web builds. Real
// headless Chromium (playwright) driving each game's actual index.html,
// with real keydown events dispatched by the browser itself -- not the
// synthesized JS objects each game's web/smoke_test.mjs uses, which only
// proves the .wasm runs without throwing, not that it looks right.
//
// Prerequisites (one-time):
//   npm install                    # from this directory
//   npx playwright install chromium
//
// Prerequisites (every run -- wasm-bindgen output isn't committed, see
// each game's web/README.md):
//   cargo build -p loom-knit -p loom-match3 -p loom-merge2 -p loom-picross \
//     --lib --target wasm32-unknown-unknown --release
//   wasm-bindgen --target web --out-dir crates/loom-knit/web/pkg    target/wasm32-unknown-unknown/release/knitui.wasm
//   wasm-bindgen --target web --out-dir crates/loom-match3/web/pkg target/wasm32-unknown-unknown/release/m3tui.wasm
//   wasm-bindgen --target web --out-dir crates/loom-merge2/web/pkg target/wasm32-unknown-unknown/release/m2tui.wasm
//   wasm-bindgen --target web --out-dir crates/loom-picross/web/pkg target/wasm32-unknown-unknown/release/pictui.wasm
//
// Run:
//   node verify.mjs
//
// Screenshots land in ./shots/<game>_<step>.png (gitignored) for manual
// visual review -- this script only checks for console/page errors and a
// harmless favicon 404 automatically; judging whether the board actually
// *looks* right still needs a human (or an AI that can view images) to
// look at the PNGs.

import { chromium } from 'playwright';
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import { extname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO = fileURLToPath(new URL('../..', import.meta.url));
const OUT = fileURLToPath(new URL('./shots', import.meta.url));

const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm' };

function serveDir(dir, port) {
  const server = createServer(async (req, res) => {
    let path = req.url.split('?')[0];
    if (path === '/') path = '/index.html';
    try {
      const data = await readFile(join(dir, path));
      res.writeHead(200, { 'Content-Type': MIME[extname(path)] || 'application/octet-stream' });
      res.end(data);
    } catch {
      res.writeHead(404);
      res.end('not found');
    }
  });
  return new Promise((resolve) => server.listen(port, () => resolve(server)));
}

// Each game's flow mirrors its own web/smoke_test.mjs key sequence
// (menu shape and screens differ per game -- see each Game impl's
// main_menu_items()), with `shot` labels marking the states worth a
// screenshot.
const GAMES = [
  {
    name: 'knit',
    dir: `${REPO}/crates/loom-knit/web`,
    port: 8801,
    steps: [
      { shot: '01_main_menu' },
      { press: 'Enter', shot: '02_after_enter_quick_game' },
      { press: 'ArrowRight' },
      { press: 'ArrowDown', shot: '03_playing' },
      { press: 'h', shot: '04_help' },
      { press: ' ', shot: '05_back_from_help' },
      { press: 'Enter', shot: '06_pick_up' },
      { press: 'Escape', shot: '07_back_to_menu' },
    ],
  },
  {
    name: 'match3',
    dir: `${REPO}/crates/loom-match3/web`,
    port: 8802,
    steps: [
      { shot: '01_main_menu' },
      { press: 'Enter', shot: '02_after_enter_quick_game' },
      { press: 'ArrowRight' },
      { press: 'Enter' },
      { press: 'ArrowDown' },
      { press: 'Enter', shot: '03_playing' },
      { press: 'h', shot: '04_help' },
      { press: ' ', shot: '05_back_from_help' },
      { press: 'Escape', shot: '06_back_to_menu' },
    ],
  },
  {
    name: 'merge2',
    dir: `${REPO}/crates/loom-merge2/web`,
    port: 8803,
    steps: [
      { shot: '01_main_menu' },
      { press: 'Enter', shot: '02_custom_game_editor' },
      { press: 'Enter', shot: '03_playing' },
      { press: 'ArrowRight' },
      { press: 'Enter' },
      { press: 'i', shot: '04_inventory' },
      { press: 'Escape' },
      { press: 'h', shot: '05_help' },
      { press: ' ', shot: '06_back_from_help' },
      { press: 'Escape', shot: '07_back_to_menu' },
    ],
  },
  {
    name: 'picross',
    dir: `${REPO}/crates/loom-picross/web`,
    port: 8804,
    steps: [
      { shot: '01_main_menu' },
      { press: 'Enter', shot: '02_track_select' },
      { press: 'Enter', shot: '03_level_intro' },
      { press: 'Enter', shot: '04_playing' },
      { press: 'ArrowRight' },
      { press: 'ArrowDown' },
      { press: 'Enter', shot: '05_fill_cell' },
      { press: 'x', shot: '06_cross_cell' },
      { press: 'h', shot: '07_help' },
      { press: ' ', shot: '08_back_from_help' },
      { press: 'Escape', shot: '09_back_to_menu' },
    ],
  },
];

await mkdir(OUT, { recursive: true });

const browser = await chromium.launch();
let failures = 0;

for (const game of GAMES) {
  console.log(`\n=== ${game.name} ===`);
  const server = await serveDir(game.dir, game.port);
  const page = await browser.newPage({ viewport: { width: 1000, height: 900 } });
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('response', (r) => {
    // The browser always requests /favicon.ico automatically; none of
    // these apps serve one, so that single 404 is expected noise, not a
    // real failure -- everything else returning >=400 is a real bug.
    if (r.status() >= 400 && !r.url().endsWith('/favicon.ico')) {
      errors.push(`HTTP ${r.status()}: ${r.url()}`);
    }
  });

  await page.goto(`http://localhost:${game.port}/`);
  await page.waitForSelector('canvas#game');
  await page.waitForTimeout(400); // let init() + first requestAnimationFrame land

  for (const step of game.steps) {
    if (step.press) {
      await page.keyboard.press(step.press);
      await page.waitForTimeout(150);
    }
    if (step.shot) {
      await page.screenshot({ path: join(OUT, `${game.name}_${step.shot}.png`) });
      console.log(`  shot: ${step.shot}`);
    }
  }

  if (errors.length > 0) {
    failures++;
    console.error(`  ERRORS:\n${errors.map((e) => '    ' + e).join('\n')}`);
  } else {
    console.log('  no console/page errors');
  }

  await page.close();
  server.close();
}

await browser.close();

if (failures > 0) {
  console.error(`\n${failures} game(s) had console/page errors.`);
  process.exit(1);
}
console.log('\nAll games ran without console/page errors. Screenshots in ./shots -- review them by eye.');
