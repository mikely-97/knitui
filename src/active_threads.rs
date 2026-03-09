// ./src/lib/board_entity.rs

use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

pub struct Thread {
    pub color: Color,
    pub status: u16,
    /// True when this thread was picked up from a KeyThread board cell.
    /// The key is consumed on the first successful yarn match.
    pub has_key: bool,
}

impl Thread {
    pub fn knit_on(&mut self) {
        self.status += 1;
    }
}

impl fmt::Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Key threads show 'k' until the key is consumed, then show progress.
        let ch = if self.has_key {
            'k'
        } else {
            match self.status {
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
    fn test_thread_creation() {
        let thread = Thread {
            color: Color::Red,
            status: 1,
            has_key: false,
        };

        assert_eq!(thread.color, Color::Red);
        assert_eq!(thread.status, 1);
        assert!(!thread.has_key);
    }

    #[test]
    fn test_key_thread_creation() {
        let thread = Thread {
            color: Color::Blue,
            status: 1,
            has_key: true,
        };

        assert!(thread.has_key);
        assert_eq!(thread.color, Color::Blue);
    }

    #[test]
    fn test_knit_on_increments_status() {
        let mut thread = Thread {
            color: Color::Blue,
            status: 1,
            has_key: false,
        };

        thread.knit_on();
        assert_eq!(thread.status, 2);

        thread.knit_on();
        assert_eq!(thread.status, 3);

        thread.knit_on();
        assert_eq!(thread.status, 4);
    }

    #[test]
    fn test_knit_on_multiple_times() {
        let mut thread = Thread {
            color: Color::Green,
            status: 1,
            has_key: false,
        };

        for _ in 0..5 {
            thread.knit_on();
        }

        assert_eq!(thread.status, 6);
    }

    #[test]
    fn test_thread_display_format() {
        let thread1 = Thread { color: Color::Red,    status: 1, has_key: false };
        let thread2 = Thread { color: Color::Blue,   status: 2, has_key: false };
        let thread3 = Thread { color: Color::Green,  status: 3, has_key: false };
        let thread_key     = Thread { color: Color::Yellow, status: 1, has_key: true  };
        let thread_unknown = Thread { color: Color::Yellow, status: 5, has_key: false };

        let _ = format!("{}", thread1);
        let _ = format!("{}", thread2);
        let _ = format!("{}", thread3);
        let _ = format!("{}", thread_key);
        let _ = format!("{}", thread_unknown);
    }

    #[test]
    fn test_thread_color_preserved_after_knit() {
        let mut thread = Thread {
            color: Color::Magenta,
            status: 1,
            has_key: false,
        };

        thread.knit_on();
        thread.knit_on();

        assert_eq!(thread.color, Color::Magenta);
    }
}
