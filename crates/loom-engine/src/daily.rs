pub fn daily_seed() -> u64 {
    // Get current date components
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    // Approximate: seconds since epoch / 86400 = days since epoch
    let days = secs / 86400;
    // Use day as seed (unique per day)
    days
}

pub fn today_string() -> String {
    // YYYY-MM-DD approximation from unix time
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days = secs / 86400;
    let y = 1970 + days / 365;
    format!("{}-daily", y) // simplified; good enough for display
}
