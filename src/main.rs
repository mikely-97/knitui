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

use knitui::game_board::{GameBoard};
use knitui::board_entity::BoardEntity;
use knitui::yarn::{Yarn};

use knitui::palette::{select_palette, ColorMode};
use knitui::active_threads::Thread;

// TODO: remove those after everything's ready for the render
// these are the constants to support rendering

const yarn_offset: u16 = 6; 
const active_offset: u16 = 1+1;
const minimal_y: u16 = yarn_offset+active_offset;

// TODO: remove those after the game is made configurable
// those are the constants to use for the game generation

const board_height: u16 = 6;
const board_width: u16 = 6;
const color_number: u16 = 6;
const color_mode: ColorMode = ColorMode::Dark;
const active_threads_limit: u16 = 7;
const knit_volume: u16 = 3;
const yarn_lines: u16 = 4;
const obstacle_percentage: u16 = 5;
const visible_patches: u16 = 6;






fn render(mut stdout: &Stdout, game_board: &GameBoard, active_threads: &Vec<Thread>, yarn: &Yarn, x: u16, y: u16) -> io::Result<()>
{
    stdout.queue(Hide);
    stdout.execute(Clear(ClearType::All))?.execute(Clear(ClearType::Purge));
    let vertical_size = game_board.height;
    let horizontal_size = game_board.width;
    // TODO: render yarn
    // stdout.queue(MoveTo(0, yarn_offset));
    stdout.queue(MoveTo(0, 0));
    stdout.queue(Print(yarn));

    // render active threads
    for thread in active_threads{
        stdout.queue(Print(&thread));
    }
    stdout.queue(Print("\n\r"));
    
    // render game board
    for thread_row in &game_board.board {
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
    // 0 - preparing the terminal
    // getting an output thread
    let mut stdout = stdout();
    // entering a new clean screen
    execute!(stdout, EnterAlternateScreen)?;
    // preventing unauthorized clicks
    enable_raw_mode()?;

    
    // 1 - preparing the assets
    // TODO: create as a separate struct (isn't urgent, it's simple enough)
    // those are the threads that we've selected and are being knitted
    let mut active_threads: Vec<Thread> = Vec::new();
    // select the palette
    let selected_palette = select_palette(color_mode, color_number);
    // generate the game board
    let mut game_board = GameBoard::make_random(
        board_height,
        board_width,
        &selected_palette,
        obstacle_percentage,
        knit_volume,
    );
    // TODO: generate the yarn
    let mut yarn = Yarn::make_from_color_counter(game_board.count_knits(), yarn_lines, visible_patches);
    // TODO: check if the game is solvable
    // render the game
    render(&stdout, &game_board, &active_threads, &yarn, 0, 0);

    
    

    let (mut x, mut y) = position()?;
    

    loop {
        // `poll()` waits for an `Event` for a given time period
        if poll(Duration::from_millis(500))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Left => x=x.saturating_sub(1),
                    KeyCode::Right => x=min(x+1, game_board.width-1),
                    KeyCode::Up => y=max(yarn_offset+active_offset, y.saturating_sub(1)),
                    KeyCode::Down => y=min(y+1,game_board.height+yarn_offset+active_offset-1),
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if let BoardEntity::Thread(color) = (game_board.board[(y-minimal_y) as usize][x as usize]){
                            active_threads.push(Thread { color: color, status: 1 });
                            game_board.board[(y-minimal_y) as usize][x as usize] = BoardEntity::Void;
                            render(&stdout, &game_board, &active_threads, &yarn, x, y);
                        }
                    },
                    KeyCode::Backspace => {
                        yarn.process_sequence(&mut active_threads);
                        active_threads.retain(|x| x.status <= knit_volume);
                        render(&stdout, &game_board, &active_threads, &yarn, x, y);
                    },
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
