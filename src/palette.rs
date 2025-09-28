// tbh it's vibe palette selection, mb will rework later lol 
// also i don't know much about color blindness, so i just hope the ai knows what it's talking about 
use rand::prelude::IndexedRandom;
use rand::seq::IteratorRandom;
use crossterm::style::Color;


// <VIBE_CODE>
// dark terminal palette
const DARK_PALETTE: [Color; 8] = [
    Color::White,
    Color::Cyan,
    Color::Green,
    Color::Yellow,
    Color::Magenta,
    Color::Red,
    Color::Blue,
    Color::DarkGrey,
];

// light terminal palette
const LIGHT_PALETTE: [Color; 8] = [
    Color::Black,
    Color::Blue,
    Color::DarkRed,
    Color::DarkGreen,
    Color::DarkMagenta,
    Color::DarkCyan,
    Color::DarkYellow,
    Color::Grey,
];

// "colorblind" greyscale palette
const GREY_PALETTE: [Color; 8] = [
    Color::Black,
    Color::DarkGrey,
    Color::Grey,
    Color::White,
    Color::AnsiValue(8),  // dim grey
    Color::AnsiValue(7),  // light grey
    Color::AnsiValue(15), // bright white
    Color::AnsiValue(0),  // darkest black
];
// </VIBE_CODE>

pub enum ColorMode{
    Bright,
    Dark,
    Colorblind
}

pub fn select_palette(mode: ColorMode, color_number: u16) -> Vec<Color>{
    let mut rng = rand::rng();
    match mode {
        ColorMode::Bright =>  LIGHT_PALETTE.choose_multiple(&mut rng, color_number as usize),
        ColorMode::Dark =>  DARK_PALETTE.choose_multiple(&mut rng, color_number as usize),
        ColorMode::Colorblind =>  GREY_PALETTE.choose_multiple(&mut rng, color_number as usize),
    }
    .cloned()
    .collect()
}
