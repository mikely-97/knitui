use std::path::PathBuf;
use std::fs;
use rand::Rng;

const FALLBACK_QUOTE: &str = "You are watching a fake ad. Touch grass.";

/// Load ad quotes from the configured file path.
/// Returns a Vec of quote strings. If the file is missing or empty,
/// returns a single fallback quote.
pub fn load_quotes(ad_file: &Option<PathBuf>, config_dir: &str) -> Vec<String> {
    let path = match ad_file {
        Some(p) => p.clone(),
        None => {
            let config_dir_path = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."));
            config_dir_path.join(config_dir).join("ads.txt")
        }
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![FALLBACK_QUOTE.to_string()],
    };

    let quotes: Vec<String> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect();

    if quotes.is_empty() {
        vec![FALLBACK_QUOTE.to_string()]
    } else {
        quotes
    }
}

/// Pick a random quote from the list.
pub fn random_quote(quotes: &[String]) -> &str {
    let mut rng = rand::rng();
    let idx = rng.random_range(0..quotes.len());
    &quotes[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_quotes_missing_file_returns_fallback() {
        let path = Some(PathBuf::from("/nonexistent/path/ads.txt"));
        let quotes = load_quotes(&path, "test");
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0], FALLBACK_QUOTE);
    }

    #[test]
    fn load_quotes_none_path_uses_default() {
        let quotes = load_quotes(&None, "loom_test");
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0], FALLBACK_QUOTE);
    }

    #[test]
    fn load_quotes_parses_file_skipping_comments_and_blanks() {
        let dir = std::env::temp_dir().join("loom_test_ads");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test_ads.txt");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "# This is a comment").unwrap();
        writeln!(f, "").unwrap();
        writeln!(f, "First quote").unwrap();
        writeln!(f, "  # Indented comment").unwrap();
        writeln!(f, "Second quote").unwrap();
        drop(f);

        let quotes = load_quotes(&Some(file_path.clone()), "test");
        assert_eq!(quotes, vec!["First quote", "Second quote"]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_quotes_empty_file_returns_fallback() {
        let dir = std::env::temp_dir().join("loom_test_empty_ads");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("empty.txt");
        std::fs::File::create(&file_path).unwrap();

        let quotes = load_quotes(&Some(file_path.clone()), "test");
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0], FALLBACK_QUOTE);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn random_quote_returns_valid_entry() {
        let quotes = vec!["Alpha".to_string(), "Beta".to_string()];
        let q = random_quote(&quotes);
        assert!(q == "Alpha" || q == "Beta");
    }
}
