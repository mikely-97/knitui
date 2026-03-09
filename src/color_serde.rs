use crossterm::style::Color;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error as DeError;

pub fn color_to_str(c: &Color) -> String {
    match c {
        Color::Reset        => "reset".into(),
        Color::Black        => "black".into(),
        Color::DarkGrey     => "darkgrey".into(),
        Color::Red          => "red".into(),
        Color::DarkRed      => "darkred".into(),
        Color::Green        => "green".into(),
        Color::DarkGreen    => "darkgreen".into(),
        Color::Yellow       => "yellow".into(),
        Color::DarkYellow   => "darkyellow".into(),
        Color::Blue         => "blue".into(),
        Color::DarkBlue     => "darkblue".into(),
        Color::Magenta      => "magenta".into(),
        Color::DarkMagenta  => "darkmagenta".into(),
        Color::Cyan         => "cyan".into(),
        Color::DarkCyan     => "darkcyan".into(),
        Color::White        => "white".into(),
        Color::Grey         => "grey".into(),
        Color::Rgb { r, g, b } => format!("rgb({r},{g},{b})"),
        Color::AnsiValue(n)    => format!("ansi({n})"),
    }
}

pub fn str_to_color(s: &str) -> Option<Color> {
    let c = match s {
        "reset"        => Color::Reset,
        "black"        => Color::Black,
        "darkgrey"     => Color::DarkGrey,
        "red"          => Color::Red,
        "darkred"      => Color::DarkRed,
        "green"        => Color::Green,
        "darkgreen"    => Color::DarkGreen,
        "yellow"       => Color::Yellow,
        "darkyellow"   => Color::DarkYellow,
        "blue"         => Color::Blue,
        "darkblue"     => Color::DarkBlue,
        "magenta"      => Color::Magenta,
        "darkmagenta"  => Color::DarkMagenta,
        "cyan"         => Color::Cyan,
        "darkcyan"     => Color::DarkCyan,
        "white"        => Color::White,
        "grey"         => Color::Grey,
        _ => {
            if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
                let p: Vec<&str> = inner.split(',').collect();
                if p.len() == 3 {
                    let r = p[0].trim().parse::<u8>().ok()?;
                    let g = p[1].trim().parse::<u8>().ok()?;
                    let b = p[2].trim().parse::<u8>().ok()?;
                    return Some(Color::Rgb { r, g, b });
                }
            }
            if let Some(inner) = s.strip_prefix("ansi(").and_then(|s| s.strip_suffix(')')) {
                return Some(Color::AnsiValue(inner.trim().parse::<u8>().ok()?));
            }
            return None;
        }
    };
    Some(c)
}

/// Use with `#[serde(with = "crate::color_serde")]` on `Color` fields.
pub fn serialize<S: Serializer>(color: &Color, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&color_to_str(color))
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
    let raw = String::deserialize(d)?;
    str_to_color(&raw).ok_or_else(|| D::Error::custom(format!("unknown color: {raw}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(c: Color) -> Color {
        str_to_color(&color_to_str(&c)).expect("roundtrip failed")
    }

    #[test]
    fn roundtrip_named() {
        for c in [Color::Red, Color::Blue, Color::Green, Color::Cyan,
                  Color::White, Color::Reset, Color::DarkGrey, Color::Magenta] {
            assert_eq!(roundtrip(c), c);
        }
    }
    #[test]
    fn roundtrip_rgb() {
        let c = Color::Rgb { r: 10, g: 200, b: 55 };
        assert_eq!(roundtrip(c), c);
    }
    #[test]
    fn roundtrip_ansi() {
        assert_eq!(roundtrip(Color::AnsiValue(42)), Color::AnsiValue(42));
    }
    #[test]
    fn unknown_returns_none() {
        assert!(str_to_color("neon-pink").is_none());
    }
}
