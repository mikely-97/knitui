/// Animation overlay for board cells during merge/spawn events.

#[derive(Clone, Copy, Debug)]
pub enum AnimKind {
    Dissolve,
    Rise,
}

#[derive(Clone, Copy, Debug)]
pub struct CellAnim {
    pub kind: AnimKind,
    /// Counts down 3 → 0; removed when it reaches 0 after a tick.
    pub frame: u8,
}
