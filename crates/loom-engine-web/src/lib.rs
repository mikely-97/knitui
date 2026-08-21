// wasm-bindgen browser frontend building blocks: a canvas-backed
// `Surface`, a browser `KeyboardEvent` -> `KeyEvent` mapper, and a
// `localStorage`-backed `Storage` impl. Each game crate's `web.rs` uses
// these to drive its full `Shell<G>` (menus, campaign, endless, options,
// help) from a `requestAnimationFrame` loop -- see e.g.
// `crates/loom-knit/web/index.html`.

use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

use loom_engine::render::{Attrs, Cell, Color, Style, Surface};
use loom_engine::input::{Key, KeyEvent, Modifiers};
use loom_engine::storage::Storage;

/// Maps `render::Color` to a CSS color string. Named variants lean on CSS's
/// own named-color table rather than a hand-rolled hex palette; `Reset`
/// resolves to the caller-supplied default since "reset to terminal default"
/// has no browser equivalent.
pub fn color_to_css(color: Color, default: &str) -> String {
    match color {
        Color::Reset => default.to_string(),
        Color::Black => "black".to_string(),
        Color::DarkGrey => "dimgray".to_string(),
        Color::Red => "red".to_string(),
        Color::DarkRed => "darkred".to_string(),
        Color::Green => "lime".to_string(),
        Color::DarkGreen => "green".to_string(),
        Color::Yellow => "yellow".to_string(),
        Color::DarkYellow => "olive".to_string(),
        Color::Blue => "royalblue".to_string(),
        Color::DarkBlue => "darkblue".to_string(),
        Color::Magenta => "magenta".to_string(),
        Color::DarkMagenta => "darkmagenta".to_string(),
        Color::Cyan => "cyan".to_string(),
        Color::DarkCyan => "darkcyan".to_string(),
        Color::White => "white".to_string(),
        Color::Grey => "lightgray".to_string(),
        Color::Rgb { r, g, b } => format!("rgb({r},{g},{b})"),
        Color::AnsiValue(n) => ansi256_to_css(n),
    }
}

/// Approximate xterm 256-color palette -> CSS rgb(). Covers the 16 base
/// colors via the named mapping above (rarely hit here since board code
/// uses named colors), the 6x6x6 color cube (16..=231), and the grayscale
/// ramp (232..=255).
fn ansi256_to_css(n: u8) -> String {
    if n < 16 {
        return color_to_css(
            [
                Color::Black, Color::DarkRed, Color::DarkGreen, Color::DarkYellow,
                Color::DarkBlue, Color::DarkMagenta, Color::DarkCyan, Color::Grey,
                Color::DarkGrey, Color::Red, Color::Green, Color::Yellow,
                Color::Blue, Color::Magenta, Color::Cyan, Color::White,
            ][n as usize],
            "white",
        );
    }
    if n >= 232 {
        let level = 8 + (n - 232) as u32 * 10;
        return format!("rgb({level},{level},{level})");
    }
    let n = n - 16;
    let steps = [0u32, 95, 135, 175, 215, 255];
    let r = steps[(n / 36) as usize];
    let g = steps[((n / 6) % 6) as usize];
    let b = steps[(n % 6) as usize];
    format!("rgb({r},{g},{b})")
}

/// Canvas-backed `Surface`. Draws a monospace character grid: each cell is
/// `cell_w` x `cell_h` CSS pixels, background painted first, then the glyph.
pub struct WasmSurface {
    ctx: CanvasRenderingContext2d,
    cols: u16,
    rows: u16,
    cell_w: f64,
    cell_h: f64,
    font_px: f64,
}

impl WasmSurface {
    /// `font_px` is the monospace font size in CSS pixels; cell width is
    /// measured from the font itself so glyphs line up on a real grid
    /// rather than an assumed aspect ratio.
    pub fn new(ctx: CanvasRenderingContext2d, cols: u16, rows: u16, font_px: f64) -> Self {
        ctx.set_font(&format!("{font_px}px monospace"));
        ctx.set_text_baseline("top");
        let cell_w = ctx.measure_text("M").map(|m| m.width()).unwrap_or(font_px * 0.6);
        let cell_h = font_px * 1.2;
        Self { ctx, cols, rows, cell_w, cell_h, font_px }
    }

    fn set_font(&self, bold: bool) {
        let weight = if bold { "bold " } else { "" };
        self.ctx.set_font(&format!("{weight}{}px monospace", self.font_px));
    }
}

impl Surface for WasmSurface {
    fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if x >= self.cols || y >= self.rows {
            return;
        }
        let px = x as f64 * self.cell_w;
        let py = y as f64 * self.cell_h;

        let reverse = cell.style.attrs.contains(Attrs::REVERSE);
        let (fg, bg) = if reverse {
            (cell.style.bg, cell.style.fg)
        } else {
            (cell.style.fg, cell.style.bg)
        };

        if bg != Color::Reset || reverse {
            self.ctx.set_fill_style_str(&color_to_css(bg, "black"));
            self.ctx.fill_rect(px, py, self.cell_w, self.cell_h);
        }

        self.set_font(cell.style.attrs.contains(Attrs::BOLD));
        self.ctx.set_global_alpha(if cell.style.attrs.contains(Attrs::DIM) { 0.6 } else { 1.0 });
        self.ctx.set_fill_style_str(&color_to_css(fg, "white"));
        let _ = self.ctx.fill_text(&cell.glyph.to_string(), px, py);
        self.ctx.set_global_alpha(1.0);
    }

    fn clear(&mut self, style: Style) {
        self.ctx.set_fill_style_str(&color_to_css(style.bg, "black"));
        self.ctx.fill_rect(0.0, 0.0, self.cols as f64 * self.cell_w, self.rows as f64 * self.cell_h);
    }
}

/// Maps a browser `KeyboardEvent` to the engine's portable `KeyEvent`.
/// Returns `None` for keys with no engine-level meaning (matches the
/// deliberately minimal `Key` enum -- no F-keys, no paste, no resize).
pub fn key_from_event(event: &web_sys::KeyboardEvent) -> Option<KeyEvent> {
    let key = match event.key().as_str() {
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "Enter" => Key::Enter,
        "Escape" => Key::Esc,
        s => {
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Key::Char(c),
                _ => return None,
            }
        }
    };

    let mut mods = Modifiers::empty();
    if event.ctrl_key() { mods |= Modifiers::CTRL; }
    if event.shift_key() { mods |= Modifiers::SHIFT; }
    if event.alt_key() { mods |= Modifiers::ALT; }

    Some(KeyEvent::with_mods(key, mods))
}

/// `localStorage`-backed `Storage`. Namespace and key are joined into one
/// flat localStorage key since `localStorage` itself has no notion of
/// namespacing.
pub struct WebStorage;

impl WebStorage {
    fn storage_key(namespace: &str, key: &str) -> String {
        format!("{namespace}:{key}")
    }

    fn local_storage(&self) -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }
}

impl Storage for WebStorage {
    fn load(&self, namespace: &str, key: &str) -> Option<String> {
        self.local_storage()?.get_item(&Self::storage_key(namespace, key)).ok().flatten()
    }

    fn save(&self, namespace: &str, key: &str, value: &str) {
        if let Some(storage) = self.local_storage() {
            let _ = storage.set_item(&Self::storage_key(namespace, key), value);
        }
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
