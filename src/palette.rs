// tbh it's vibe palette selection, mb will rework later lol 
// also i don't know much about color blindness, so i just hope the ai knows what it's talking about 
use rand::prelude::IndexedRandom;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_palette_dark_returns_correct_count() {
        let colors = select_palette(ColorMode::Dark, 3);
        assert_eq!(colors.len(), 3);
    }

    #[test]
    fn test_select_palette_bright_returns_correct_count() {
        let colors = select_palette(ColorMode::Bright, 5);
        assert_eq!(colors.len(), 5);
    }

    #[test]
    fn test_select_palette_colorblind_returns_correct_count() {
        let colors = select_palette(ColorMode::Colorblind, 4);
        assert_eq!(colors.len(), 4);
    }

    #[test]
    fn test_select_palette_single_color() {
        let colors = select_palette(ColorMode::Dark, 1);
        assert_eq!(colors.len(), 1);
    }

    #[test]
    fn test_select_palette_max_colors() {
        let colors = select_palette(ColorMode::Dark, 8);
        assert_eq!(colors.len(), 8);
    }

    #[test]
    fn test_select_palette_dark_contains_valid_colors() {
        let colors = select_palette(ColorMode::Dark, 8);

        // All colors should be from DARK_PALETTE
        for color in &colors {
            assert!(DARK_PALETTE.contains(color));
        }
    }

    #[test]
    fn test_select_palette_bright_contains_valid_colors() {
        let colors = select_palette(ColorMode::Bright, 8);

        // All colors should be from LIGHT_PALETTE
        for color in &colors {
            assert!(LIGHT_PALETTE.contains(color));
        }
    }

    #[test]
    fn test_select_palette_colorblind_contains_valid_colors() {
        let colors = select_palette(ColorMode::Colorblind, 8);

        // All colors should be from GREY_PALETTE
        for color in &colors {
            assert!(GREY_PALETTE.contains(color));
        }
    }

    #[test]
    fn test_select_palette_no_duplicates() {
        let colors = select_palette(ColorMode::Dark, 5);

        // Since we're choosing from a palette, there should be no duplicates
        let mut unique_colors = colors.clone();
        unique_colors.sort_by_key(|c| format!("{:?}", c));
        unique_colors.dedup();

        assert_eq!(colors.len(), unique_colors.len());
    }

    #[test]
    fn test_palette_constants_have_8_colors() {
        assert_eq!(DARK_PALETTE.len(), 8);
        assert_eq!(LIGHT_PALETTE.len(), 8);
        assert_eq!(GREY_PALETTE.len(), 8);
    }

    #[test]
    fn test_select_palette_zero_colors() {
        let colors = select_palette(ColorMode::Dark, 0);
        assert_eq!(colors.len(), 0);
    }
}
