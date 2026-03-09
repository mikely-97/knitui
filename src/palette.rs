// tbh it's vibe palette selection, mb will rework later lol 
// also i don't know much about color blindness, so i just hope the ai knows what it's talking about 
use rand::prelude::IndexedRandom;
use crossterm::style::Color;


// <VIBE_CODE>
// dark terminal palette (ANSI — terminal theme can remap these)
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

// light terminal palette (ANSI)
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

// "colorblind" greyscale palette (ANSI)
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

// RGB palettes — exact colors immune to terminal theme overrides
const DARK_RGB_PALETTE: [Color; 8] = [
    Color::Rgb { r: 255, g: 255, b: 255 }, // white
    Color::Rgb { r:   0, g: 200, b: 200 }, // cyan
    Color::Rgb { r:   0, g: 200, b:   0 }, // green
    Color::Rgb { r: 200, g: 200, b:   0 }, // yellow
    Color::Rgb { r: 200, g:   0, b: 200 }, // magenta
    Color::Rgb { r: 200, g:   0, b:   0 }, // red
    Color::Rgb { r:  80, g:  80, b: 255 }, // blue
    Color::Rgb { r: 100, g: 100, b: 100 }, // dark grey
];

const LIGHT_RGB_PALETTE: [Color; 8] = [
    Color::Rgb { r:   0, g:   0, b:   0 }, // black
    Color::Rgb { r:   0, g:   0, b: 200 }, // blue
    Color::Rgb { r: 150, g:   0, b:   0 }, // dark red
    Color::Rgb { r:   0, g: 150, b:   0 }, // dark green
    Color::Rgb { r: 150, g:   0, b: 150 }, // dark magenta
    Color::Rgb { r:   0, g: 150, b: 150 }, // dark cyan
    Color::Rgb { r: 150, g: 150, b:   0 }, // dark yellow
    Color::Rgb { r: 180, g: 180, b: 180 }, // grey
];

const GREY_RGB_PALETTE: [Color; 8] = [
    Color::Rgb { r:   0, g:   0, b:   0 }, // black
    Color::Rgb { r:  85, g:  85, b:  85 }, // dark grey
    Color::Rgb { r: 170, g: 170, b: 170 }, // grey
    Color::Rgb { r: 255, g: 255, b: 255 }, // white
    Color::Rgb { r:  50, g:  50, b:  50 }, // dim grey
    Color::Rgb { r: 192, g: 192, b: 192 }, // light grey
    Color::Rgb { r: 240, g: 240, b: 240 }, // bright white
    Color::Rgb { r:  20, g:  20, b:  20 }, // near black
];
// </VIBE_CODE>

pub enum ColorMode {
    Bright,
    Dark,
    Colorblind,
    DarkRgb,
    BrightRgb,
    ColorblindRgb,
}

pub fn select_palette(mode: ColorMode, color_number: u16) -> Vec<Color>{
    let mut rng = rand::rng();
    match mode {
        ColorMode::Bright =>  LIGHT_PALETTE.choose_multiple(&mut rng, color_number as usize),
        ColorMode::Dark =>  DARK_PALETTE.choose_multiple(&mut rng, color_number as usize),
        ColorMode::Colorblind =>  GREY_PALETTE.choose_multiple(&mut rng, color_number as usize),
        ColorMode::DarkRgb =>  DARK_RGB_PALETTE.choose_multiple(&mut rng, color_number as usize),
        ColorMode::BrightRgb =>  LIGHT_RGB_PALETTE.choose_multiple(&mut rng, color_number as usize),
        ColorMode::ColorblindRgb =>  GREY_RGB_PALETTE.choose_multiple(&mut rng, color_number as usize),
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
