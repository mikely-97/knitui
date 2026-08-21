use loom_engine::game::GameConfig;

#[derive(Clone, Debug)]
pub struct Config {
    pub scale: u16,
    pub color_mode: String,
    /// Set by `PicrossGame::campaign_config` to the selected puzzle's real
    /// dimensions, purely for `chrome::render_level_intro`'s display -- the
    /// engine itself is always built directly from the looked-up `Puzzle`,
    /// never reconstructed from this `Config` (there's no way to encode a
    /// puzzle's clues/solution as scalar config fields).
    pub board_rows: usize,
    pub board_cols: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scale: 1,
            color_mode: "dark".to_string(),
            board_rows: 10,
            board_cols: 10,
        }
    }
}

impl GameConfig for Config {
    fn board_width(&self) -> usize { self.board_cols }
    fn board_height(&self) -> usize { self.board_rows }
    fn color_count(&self) -> usize { 2 }
    fn scale(&self) -> u16 { self.scale }
    fn color_mode(&self) -> &str { &self.color_mode }
    fn set_scale(&mut self, s: u16) { self.scale = s; }
    fn set_color_mode(&mut self, m: String) { self.color_mode = m; }

    // Picross has no CustomGame screen today (puzzles are curated, not
    // randomly generated from parameters) -- see the Phase 3 scaffolding
    // survey. No fields to expose or adjust.
    fn custom_fields(&self) -> Vec<(&'static str, u16)> { Vec::new() }
    fn adjust_custom_field(&mut self, _field: usize, _delta: i16) {}
}
