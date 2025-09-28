use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

pub struct Patch {
    color: Color
}

impl fmt::Display for Patch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            '▦'.with(self.color)
        )
    }
}

pub struct Yarn {

}
