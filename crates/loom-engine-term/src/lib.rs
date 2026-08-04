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
