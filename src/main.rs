#![allow(warnings)]

use std::fmt;
use std::fmt::Display;
use std::io::{Write, stdout};
use std::io;

use crossterm::{
    ExecutableCommand, execute, queue, QueueableCommand,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, style, Attribute, Stylize},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, SetSize, enable_raw_mode, disable_raw_mode},
    cursor::{DisableBlinking, EnableBlinking, MoveTo, RestorePosition, SavePosition, MoveRight, MoveLeft, MoveDown, MoveUp, position},
    event::{self, poll, read, Event, KeyCode},
};
use std::time::Duration; 
use std::cmp::{
    min, max,
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
                1 => '|'.with(self.color),
                2 => '+'.with(self.color),
                3 => 'F'.with(self.color),
                _ => 'B'.with(self.color),
            }
        )
    }
}

fn decr_if_possible(val: u16) -> u16{
    if (val == 0) {return val} 
    else {return val-1};
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

    stdout.queue(MoveTo(0,0));
    stdout.execute(Clear(ClearType::All))?.execute(Clear(ClearType::Purge));
    stdout.execute(SetSize(4, 4));
    enable_raw_mode()?;
    for thread_row in &game_board {

        for thread in thread_row {
            stdout.queue(Print(&thread));
        }
        stdout.queue(Print('\n'));
        stdout.queue(Print('\r'));
    }

    stdout.queue(MoveTo(0,0));
    stdout.queue(EnableBlinking);
    
    
    stdout.flush();

    let (mut x, mut y) = position()?;

    loop {
        // `poll()` waits for an `Event` for a given time period
        if poll(Duration::from_millis(500))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Left => x=decr_if_possible(x),
                    KeyCode::Right => x=min(x+1, 4),
                    KeyCode::Up => y=decr_if_possible(y),
                    KeyCode::Down => y=min(y+1,4),
                    KeyCode::Esc => break,
                    _ => {},
                };
                // println!("{x}, {y}");
                stdout.execute(MoveTo(x, y));
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }



    execute!(stdout, LeaveAlternateScreen);
    disable_raw_mode()?;
    Ok(())
}
