use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimKind {
    /// Source cell in a merge/match — dissolves away.
    Dissolve,
    /// Destination cell in a merge, or a generator spawn — rises in.
    Rise,
}

#[derive(Clone, Copy, Debug)]
pub struct AnimFrame {
    pub kind: AnimKind,
    pub frame: u8, // starts at 3, counts down to 0 then removed
}

impl AnimFrame {
    pub fn dissolve() -> Self { Self { kind: AnimKind::Dissolve, frame: 3 } }
    pub fn rise() -> Self { Self { kind: AnimKind::Rise, frame: 3 } }
    pub fn rise_brief() -> Self { Self { kind: AnimKind::Rise, frame: 2 } }
}

#[derive(Clone, Debug, Default)]
pub struct AnimOverlay {
    cells: HashMap<(usize, usize), AnimFrame>,
}

impl AnimOverlay {
    pub fn new() -> Self { Self::default() }

    /// Insert a dissolve animation at pos.
    pub fn dissolve(&mut self, pos: (usize, usize)) {
        self.cells.insert(pos, AnimFrame::dissolve());
    }

    /// Insert a rise animation at pos.
    pub fn rise(&mut self, pos: (usize, usize)) {
        self.cells.insert(pos, AnimFrame::rise());
    }

    /// Insert a brief (2-frame) rise animation at pos (for generator spawns etc).
    pub fn rise_brief(&mut self, pos: (usize, usize)) {
        self.cells.insert(pos, AnimFrame::rise_brief());
    }

    /// Insert a raw frame (for backwards compat with code that used u8 directly).
    pub fn insert_raw(&mut self, pos: (usize, usize), frame: u8) {
        self.cells.insert(pos, AnimFrame { kind: AnimKind::Dissolve, frame });
    }

    /// Advance all animations by one tick. Returns true if any cells remain.
    pub fn tick(&mut self) -> bool {
        self.cells.retain(|_, f| {
            if f.frame == 0 { false } else { f.frame -= 1; true }
        });
        !self.cells.is_empty()
    }

    /// Get the current animation frame for a cell, if any.
    pub fn get(&self, pos: (usize, usize)) -> Option<AnimFrame> {
        self.cells.get(&pos).copied()
    }

    pub fn is_empty(&self) -> bool { self.cells.is_empty() }

    pub fn contains_key(&self, pos: &(usize, usize)) -> bool {
        self.cells.contains_key(pos)
    }
}
