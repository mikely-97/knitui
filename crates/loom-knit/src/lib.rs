pub mod board_entity;
pub mod game_board;
pub mod yarn;
pub mod spool;
pub mod color_counter;
pub mod config;
pub mod solvability;
pub mod engine;
pub mod renderer;
pub mod glyphs;
pub mod preset;
pub mod campaign;
pub mod campaign_levels;
pub mod endless;
pub mod blessings;
pub mod game;
#[cfg(not(target_arch = "wasm32"))]
pub mod tui;
#[cfg(target_arch = "wasm32")]
pub mod web;

// Re-export shared modules from loom-engine so existing `crate::` paths keep working.
pub use loom_engine::palette;
pub use loom_engine::color_serde;
pub use loom_engine::settings;
pub use loom_engine::ad_content;
