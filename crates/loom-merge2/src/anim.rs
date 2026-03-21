// Re-export from loom-engine for backwards compatibility.
pub use loom_engine::anim::{AnimKind, AnimFrame, AnimOverlay};

// Legacy type alias so existing code using CellAnim still compiles.
pub use loom_engine::anim::AnimFrame as CellAnim;
