use loom_engine::game::GameConfig;

#[derive(Clone, Debug)]
pub struct Config {
    pub scale: u16,
    pub color_mode: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scale: 1,
            color_mode: "dark".to_string(),
        }
    }
}

impl GameConfig for Config {
    fn board_width(&self) -> usize { 10 }
    fn board_height(&self) -> usize { 10 }
    fn color_count(&self) -> usize { 2 }
    fn scale(&self) -> u16 { self.scale }
    fn color_mode(&self) -> &str { &self.color_mode }
    fn set_scale(&mut self, s: u16) { self.scale = s; }
    fn set_color_mode(&mut self, m: String) { self.color_mode = m; }
}
