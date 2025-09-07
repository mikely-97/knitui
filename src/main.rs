use std::fmt;
use std::fmt::Display;
use std::io::{Write, stdout};

use crossterm::{
    ExecutableCommand, event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

struct Thread {
    color: Color,
    status: u8,
}

impl Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.status {
                1 => '_',
                2 => '二',
                3 => '三',
                _ => ' ',
            }
        )
    }
}

fn main() -> std::io::Result<()> {
    let A = [
        Thread {
            color: Color::Red,
            status: 3,
        },
        Thread {
            color: Color::Magenta,
            status: 2,
        },
        Thread {
            color: Color::Blue,
            status: 1,
        },
        Thread {
            color: Color::Yellow,
            status: 3,
        },
        Thread {
            color: Color::Green,
            status: 3,
        },
    ];

    // using the macro
    /*execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        SetBackgroundColor(Color::Red),
        Print("Styled text here."),
        ResetColor
    )?;*/

    // or using functions
    stdout()
        //.execute(SetForegroundColor(Color::Blue))?
        //.execute(SetBackgroundColor(Color::Red))?
        .execute(Print(&A[0]))?
        //.execute(Print(A[1]))?
        //.execute(Print(A[2]))?
        //.execute(Print(A[3]))?
        //.execute(Print(A[4]))?
        .execute(ResetColor)?;

    Ok(())
}
