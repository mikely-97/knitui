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
                1 => '1'.with(self.color),
                2 => '2'.with(self.color),
                3 => '3'.with(self.color),
                _ => '?'.with(self.color),
            }
        )
    }
}

struct Patch {
    color: Color
}

impl Display for Patch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            '▦'.with(self.color)
        )
    }
}

enum BoardEntity {
    Thread(Color),
    Obstacle,
    Void,
}

impl Display for BoardEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BoardEntity::Thread(color) => 'T'.with(*color),
                BoardEntity::Obstacle => 'X'.stylize(),
                BoardEntity::Void => ' '.stylize()
            }
        )
    }
}


fn main() -> std::io::Result<()> {

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;

    
    let mut active_threads: Vec<Thread> = Vec::new();


    let mut game_board: Vec<Vec<BoardEntity>> = vec![
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Thread(Color::Blue),
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Obstacle,
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Thread(Color::Blue),
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Thread(Color::Blue),
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Thread(Color::Blue),
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
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
                    KeyCode::Left => x=x.saturating_sub(1),
                    KeyCode::Right => x=min(x+1, 4),
                    KeyCode::Up => y=y.saturating_sub(1),
                    KeyCode::Down => y=min(y+1,4),
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if let BoardEntity::Thread(color) = (game_board[y as usize][x as usize]){
                            active_threads.push(Thread { color: color, status: 1 });
                            game_board[y as usize][x as usize] = BoardEntity::Void;
                        }
                    }
                    _ => {},
                };
                // println!("{x}, {y}");
                //stdout.execute(MoveTo(x, y));
                stdout.queue(MoveTo(0,0));
                stdout.queue(Clear(ClearType::All))?.execute(Clear(ClearType::Purge));
                for thread_row in &game_board {
                    for thread in thread_row {
                        stdout.queue(Print(&thread));
                    }
                    stdout.queue(Print('\n'));
                    stdout.queue(Print('\r'));
                };
                stdout.queue(MoveTo(x, y));
                stdout.flush();
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }



    execute!(stdout, LeaveAlternateScreen);
    disable_raw_mode()?;
    Ok(())
}
