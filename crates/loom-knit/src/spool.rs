// ./src/lib/board_entity.rs

use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

pub struct Spool {
    pub color: Color,
    pub fill: u16,
    /// True when this spool was picked up from a KeySpool board cell.
    /// The key is consumed on the first successful yarn match.
    pub has_key: bool,
}

impl Spool {
    pub fn wind(&mut self) {
        self.fill += 1;
    }
}

impl fmt::Display for Spool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Key spools show 'k' until the key is consumed, then show progress.
        let ch = if self.has_key {
            'k'
        } else {
            match self.fill {
                1 => '0',
                2 => '1',
                3 => '2',
                _ => '?',
            }
        };
        write!(f, "{}", ch.with(self.color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spool_creation() {
        let spool = Spool {
            color: Color::Red,
            fill: 1,
            has_key: false,
        };

        assert_eq!(spool.color, Color::Red);
        assert_eq!(spool.fill, 1);
        assert!(!spool.has_key);
    }

    #[test]
    fn test_key_spool_creation() {
        let spool = Spool {
            color: Color::Blue,
            fill: 1,
            has_key: true,
        };

        assert!(spool.has_key);
        assert_eq!(spool.color, Color::Blue);
    }

    #[test]
    fn test_wind_increments_fill() {
        let mut spool = Spool {
            color: Color::Blue,
            fill: 1,
            has_key: false,
        };

        spool.wind();
        assert_eq!(spool.fill, 2);

        spool.wind();
        assert_eq!(spool.fill, 3);

        spool.wind();
        assert_eq!(spool.fill, 4);
    }

    #[test]
    fn test_wind_multiple_times() {
        let mut spool = Spool {
            color: Color::Green,
            fill: 1,
            has_key: false,
        };

        for _ in 0..5 {
            spool.wind();
        }

        assert_eq!(spool.fill, 6);
    }

    #[test]
    fn test_spool_display_format() {
        let spool1 = Spool { color: Color::Red,    fill: 1, has_key: false };
        let spool2 = Spool { color: Color::Blue,   fill: 2, has_key: false };
        let spool3 = Spool { color: Color::Green,  fill: 3, has_key: false };
        let spool_key     = Spool { color: Color::Yellow, fill: 1, has_key: true  };
        let spool_unknown = Spool { color: Color::Yellow, fill: 5, has_key: false };

        let _ = format!("{}", spool1);
        let _ = format!("{}", spool2);
        let _ = format!("{}", spool3);
        let _ = format!("{}", spool_key);
        let _ = format!("{}", spool_unknown);
    }

    #[test]
    fn test_spool_color_preserved_after_wind() {
        let mut spool = Spool {
            color: Color::Magenta,
            fill: 1,
            has_key: false,
        };

        spool.wind();
        spool.wind();

        assert_eq!(spool.color, Color::Magenta);
    }
}
