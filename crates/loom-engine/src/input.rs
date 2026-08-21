// Portable, crossterm-free input types. Deliberately minimal — matches the
// confirmed real usage across all 4 games (no F-keys, mouse, paste, or
// resize handling in any GameEngine impl). Extend when a real game needs a
// real key, not before.

use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct Modifiers: u8 {
        const CTRL  = 0b001;
        const SHIFT = 0b010;
        const ALT   = 0b100;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Enter,
    Esc,
}

/// Also the FFI key-input payload (`loom-engine-capi` deserializes this
/// directly from JSON) -- `derive(Serialize, Deserialize)` here is load-
/// bearing for that boundary, not just a convenience.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key: Key,
    pub mods: Modifiers,
}

impl KeyEvent {
    pub fn new(key: Key) -> Self {
        Self { key, mods: Modifiers::empty() }
    }

    pub fn with_mods(key: Key, mods: Modifiers) -> Self {
        Self { key, mods }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_modifiers() {
        let ev = KeyEvent::new(Key::Char('a'));
        assert_eq!(ev.key, Key::Char('a'));
        assert_eq!(ev.mods, Modifiers::empty());
    }

    #[test]
    fn with_mods_sets_modifiers() {
        let ev = KeyEvent::with_mods(Key::Up, Modifiers::CTRL);
        assert!(ev.mods.contains(Modifiers::CTRL));
        assert!(!ev.mods.contains(Modifiers::SHIFT));
    }
}
