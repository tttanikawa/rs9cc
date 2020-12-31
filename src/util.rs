pub fn split_digit(s: &str) -> usize {
    s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len())
}
