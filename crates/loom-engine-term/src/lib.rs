// Crossterm-backed terminal frontend support. Split out of `loom-engine` so
// the core crate stays crossterm-free and portable to non-terminal targets
// (web, FFI hosts). The crossterm `Surface` impl (Phase 1) lands here too.

use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    cursor,
};
use std::io;
use std::panic;

/// Initialize the terminal for TUI use: raw mode, alternate screen, hidden cursor.
/// Sets a panic hook that restores the terminal before printing the panic.
pub fn init() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
    let orig = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore();
        orig(info);
    }));
    Ok(())
}

/// Restore the terminal to its original state.
pub fn restore() -> io::Result<()> {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
    Ok(())
}

/// Run `f` with the terminal initialized, restoring on exit or panic.
pub fn run<F: FnOnce() -> io::Result<()>>(f: F) -> io::Result<()> {
    init()?;
    let result = f();
    restore()?;
    result
}

// ── Surface impl ─────────────────────────────────────────────────────────

use crossterm::{
    QueueableCommand,
    style::{Attribute, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    cursor::MoveTo,
};
use loom_engine::render::{Attrs, Cell, Color, Style, Surface};
use std::io::{Stdout, Write};

/// Translate a portable `render::Color` to crossterm's own color type.
/// Kept public so frontends/tools that talk to crossterm directly (rather
/// than through `Surface`) can reuse the same mapping.
pub fn color_to_crossterm(c: Color) -> crossterm::style::Color {
    use crossterm::style::Color as CC;
    match c {
        Color::Reset => CC::Reset,
        Color::Black => CC::Black,
        Color::DarkGrey => CC::DarkGrey,
        Color::Red => CC::Red,
        Color::DarkRed => CC::DarkRed,
        Color::Green => CC::Green,
        Color::DarkGreen => CC::DarkGreen,
        Color::Yellow => CC::Yellow,
        Color::DarkYellow => CC::DarkYellow,
        Color::Blue => CC::Blue,
        Color::DarkBlue => CC::DarkBlue,
        Color::Magenta => CC::Magenta,
        Color::DarkMagenta => CC::DarkMagenta,
        Color::Cyan => CC::Cyan,
        Color::DarkCyan => CC::DarkCyan,
        Color::White => CC::White,
        Color::Grey => CC::Grey,
        Color::Rgb { r, g, b } => CC::Rgb { r, g, b },
        Color::AnsiValue(v) => CC::AnsiValue(v),
    }
}

fn write_run(stdout: &mut Stdout, x: u16, y: u16, text: &str, style: Style) -> io::Result<()> {
    stdout.queue(MoveTo(x, y))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    if style.fg != Color::Reset {
        stdout.queue(SetForegroundColor(color_to_crossterm(style.fg)))?;
    }
    if style.bg != Color::Reset {
        stdout.queue(SetBackgroundColor(color_to_crossterm(style.bg)))?;
    }
    if style.attrs.contains(Attrs::BOLD) { stdout.queue(SetAttribute(Attribute::Bold))?; }
    if style.attrs.contains(Attrs::UNDERLINE) { stdout.queue(SetAttribute(Attribute::Underlined))?; }
    if style.attrs.contains(Attrs::REVERSE) { stdout.queue(SetAttribute(Attribute::Reverse))?; }
    if style.attrs.contains(Attrs::DIM) { stdout.queue(SetAttribute(Attribute::Dim))?; }
    stdout.queue(Print(text))?;
    Ok(())
}

/// A crossterm-backed `Surface` that writes straight to a live terminal.
///
/// `begin`/`finish` bracket a synchronized-update frame (hide cursor, clear
/// screen, flush at the end) — for a screen that owns and clears its frame.
/// `new`/`done` wrap a plain draw with no framing, for overlays drawn
/// mid-frame by a caller that manages its own clear/flush (e.g. the
/// celebration animation, drawn on top of an already-rendered board).
pub struct TermSurface<'a> {
    stdout: &'a mut Stdout,
    err: io::Result<()>,
}

impl<'a> TermSurface<'a> {
    pub fn new(stdout: &'a mut Stdout) -> Self {
        Self { stdout, err: Ok(()) }
    }

    pub fn begin(stdout: &'a mut Stdout) -> io::Result<Self> {
        stdout.queue(crossterm::terminal::BeginSynchronizedUpdate)?;
        stdout.queue(crossterm::cursor::Hide)?;
        stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        Ok(Self::new(stdout))
    }

    /// End a framed draw: closes the synchronized update and flushes.
    pub fn finish(self) -> io::Result<()> {
        self.err?;
        self.stdout.queue(crossterm::terminal::EndSynchronizedUpdate)?;
        self.stdout.flush()
    }

    /// End a plain (unframed) draw: just propagates any write error.
    pub fn done(self) -> io::Result<()> {
        self.err
    }
}

impl<'a> Surface for TermSurface<'a> {
    fn size(&self) -> (u16, u16) {
        crossterm::terminal::size().unwrap_or((80, 24))
    }

    fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if self.err.is_err() { return; }
        let mut buf = [0u8; 4];
        let s = cell.glyph.encode_utf8(&mut buf);
        if let Err(e) = write_run(self.stdout, x, y, s, cell.style) {
            self.err = Err(e);
        }
    }

    fn clear(&mut self, _style: Style) {
        if self.err.is_err() { return; }
        if let Err(e) = self.stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All)) {
            self.err = Err(e);
        }
    }

    fn print(&mut self, x: u16, y: u16, text: &str, style: Style) {
        if self.err.is_err() { return; }
        if let Err(e) = write_run(self.stdout, x, y, text, style) {
            self.err = Err(e);
        }
    }
}
