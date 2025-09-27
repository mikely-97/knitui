#![allow(warnings)]

use std::fmt;
use std::fmt::Display;
use std::io::{Write, stdout, Stdout};
use std::io;

use crossterm::{
    ExecutableCommand, execute, queue, QueueableCommand,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, style, Attribute, Stylize},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, SetSize, enable_raw_mode, disable_raw_mode},
    cursor::{DisableBlinking, EnableBlinking, MoveTo, RestorePosition, SavePosition, Hide, Show, position},
    event::{self, poll, read, Event, KeyCode},
};
use std::time::Duration; 
use std::cmp::{
    min, max,
};

use knitui::game_board::make_game_board;
use knitui::board_entity::BoardEntity;

const yarn_offset: u16 = 4+1; 
const active_offset: u16 = 1+1;
const minimal_y: u16 = yarn_offset+active_offset;

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



fn render(mut stdout: &Stdout, game_board: &Vec<Vec<BoardEntity>>, active_threads: &Vec<Thread>, x: u16, y: u16) -> io::Result<()>
{
    stdout.queue(Hide);
    stdout.execute(Clear(ClearType::All))?.execute(Clear(ClearType::Purge));
    let vertical_size = (game_board.len() as u16);
    let horizontal_size =(game_board[0].len() as u16);
    // TODO: render yarn
    stdout.queue(MoveTo(0, yarn_offset));

    // render active threads
    for thread in active_threads{
        stdout.queue(Print(&thread));
    }
    stdout.queue(Print("\n\r"));
    
    // render game board
    for thread_row in game_board {
        stdout.queue(Print("\n\r"));
        for thread in thread_row {
            stdout.queue(Print(&thread));
        }
        
    }
    let (mut size_x, mut size_y) = position()?;
    stdout.queue(SetSize(size_x, size_y));
    stdout.queue(MoveTo(x, max(y, minimal_y)));


    stdout.queue(Show);
    let result = stdout.flush();
    
    
    return result;
}




fn main() -> std::io::Result<()> {

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    

    
    let mut active_threads: Vec<Thread> = Vec::new();

    let mut game_board = make_game_board();

    render(&stdout, &game_board, &active_threads, 0, 0);

    
    

    let (mut x, mut y) = position()?;
    

    loop {
        // `poll()` waits for an `Event` for a given time period
        if poll(Duration::from_millis(500))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Left => x=x.saturating_sub(1),
                    KeyCode::Right => x=min(x+1, (game_board[0].len() as u16)-1),
                    KeyCode::Up => y=max(yarn_offset+active_offset, y.saturating_sub(1)),
                    KeyCode::Down => y=min(y+1,(game_board.len() as u16)+yarn_offset+active_offset-1),
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if let BoardEntity::Thread(color) = (game_board[(y-minimal_y) as usize][x as usize]){
                            active_threads.push(Thread { color: color, status: 1 });
                            game_board[(y-minimal_y) as usize][x as usize] = BoardEntity::Void;
                            render(&stdout, &game_board, &active_threads, x, y);
                        }
                    }
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
