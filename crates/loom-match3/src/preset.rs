use crate::config::Config;

pub struct Preset {
    pub name: &'static str,
    pub board_height: u16,
    pub board_width: u16,
    pub color_number: u8,
    pub move_limit: u32,
    pub special_tile_pct: u16,
}

impl Preset {
    pub fn to_config(&self, base: &Config) -> Config {
        let mut cfg = base.clone();
        cfg.board_height     = self.board_height;
        cfg.board_width      = self.board_width;
        cfg.color_number     = self.color_number;
        cfg.move_limit       = self.move_limit;
        cfg.special_tile_pct = self.special_tile_pct;
        cfg
    }
}

pub const PRESETS: &[Preset] = &[
    Preset { name: "Easy",   board_height: 6,  board_width: 6,  color_number: 4, move_limit: 40, special_tile_pct: 0  },
    Preset { name: "Medium", board_height: 8,  board_width: 8,  color_number: 6, move_limit: 30, special_tile_pct: 5  },
    Preset { name: "Hard",   board_height: 10, board_width: 10, color_number: 7, move_limit: 25, special_tile_pct: 15 },
];
