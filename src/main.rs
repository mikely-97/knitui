#![allow(warnings)]

use std::fmt;
use std::fmt::Display;
use std::io::{Write, stdout};
use std::io;

use crossterm::{
    ExecutableCommand, execute, queue, QueueableCommand,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, style, Attribute, Stylize},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, SetSize},
    cursor::{DisableBlinking, EnableBlinking, MoveTo, RestorePosition, SavePosition},
    event::{self, poll, read, Event},
};
use std::time::Duration; 



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
                1 => '0'.with(self.color),
                2 => '1'.with(self.color),
                3 => '2'.with(self.color),
                _ => '3'.with(self.color),
            }
        )
    }
}

fn main() -> std::io::Result<()> {

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;

    
    let mut active_threads: Vec<Thread> = Vec::new();


    let mut game_board: Vec<Vec<Thread>> = vec![
        vec![
        Thread {color: Color::Red, status: 0},
        Thread {color: Color::Magenta, status: 0},
        Thread {color: Color::Blue, status: 0},
        Thread {color: Color::Yellow, status: 0},
        Thread {color: Color::Green, status: 0},
        ],
        vec![
        Thread {color: Color::Red, status: 0},
        Thread {color: Color::Magenta, status: 0},
        Thread {color: Color::Blue, status: 0},
        Thread {color: Color::Yellow, status: 0},
        Thread {color: Color::Green, status: 0},
        ],
        vec![
        Thread {color: Color::Red, status: 0},
        Thread {color: Color::Magenta, status: 0},
        Thread {color: Color::Blue, status: 0},
        Thread {color: Color::Yellow, status: 0},
        Thread {color: Color::Green, status: 0},
        ],
        vec![
        Thread {color: Color::Red, status: 0},
        Thread {color: Color::Magenta, status: 0},
        Thread {color: Color::Blue, status: 0},
        Thread {color: Color::Yellow, status: 0},
        Thread {color: Color::Green, status: 0},
        ],
        vec![
        Thread {color: Color::Red, status: 0},
        Thread {color: Color::Magenta, status: 0},
        Thread {color: Color::Blue, status: 0},
        Thread {color: Color::Yellow, status: 0},
        Thread {color: Color::Green, status: 0},
        ],
    ];

    
    stdout.execute(Clear(ClearType::All))?.execute(Clear(ClearType::Purge));
    stdout.execute(SetSize(22, 22));
    for thread_row in &game_board {

        for thread in thread_row {
            stdout.queue(Print(&thread));
        }
        stdout.queue(Print('\n'));
    }

    stdout.queue(MoveTo(5,5));
    stdout.queue(EnableBlinking);
    
    
    stdout.flush();

    loop {
        // `poll()` waits for an `Event` for a given time period
        if poll(Duration::from_millis(500))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match read()? {
                Event::FocusGained => println!("FocusGained"),
                Event::FocusLost => println!("FocusLost"),
                Event::Key(event) => println!("{:?}", event),
                Event::Mouse(event) => println!("{:?}", event),
                // #[cfg(feature = "bracketed-paste")]
                Event::Paste(data) => println!("Pasted {:?}", data),
                Event::Resize(width, height) => println!("New size {}x{}", width, height),
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }



    execute!(stdout, LeaveAlternateScreen);
    Ok(())
}
