// ./src/lib/board_entity.rs

use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

pub struct Thread {
    pub color: Color,
    pub status: u16,
}

impl Thread{
    pub fn knit_on(&mut self){
        self.status += 1;
    }
}

impl fmt::Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.status {
                1 => '0'.with(self.color),
                2 => '1'.with(self.color),
                3 => '2'.with(self.color),
                _ => '?'.with(self.color),
            }
        )
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
        };

        assert_eq!(thread.color, Color::Red);
        assert_eq!(thread.status, 1);
    }

    #[test]
    fn test_knit_on_increments_status() {
        let mut thread = Thread {
            color: Color::Blue,
            status: 1,
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
        };

        for _ in 0..5 {
            thread.knit_on();
        }

        assert_eq!(thread.status, 6);
    }

    #[test]
    fn test_thread_display_format() {
        let thread1 = Thread { color: Color::Red, status: 1 };
        let thread2 = Thread { color: Color::Blue, status: 2 };
        let thread3 = Thread { color: Color::Green, status: 3 };
        let thread_unknown = Thread { color: Color::Yellow, status: 5 };

        // Just verify that format! doesn't panic
        let _ = format!("{}", thread1);
        let _ = format!("{}", thread2);
        let _ = format!("{}", thread3);
        let _ = format!("{}", thread_unknown);
    }

    #[test]
    fn test_thread_color_preserved_after_knit() {
        let mut thread = Thread {
            color: Color::Magenta,
            status: 1,
        };

        thread.knit_on();
        thread.knit_on();

        assert_eq!(thread.color, Color::Magenta);
    }
}

