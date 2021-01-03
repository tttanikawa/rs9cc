pub fn split_digits(s: &str) -> usize {
    s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len())
}

pub fn split_alphabets(s: &str) -> usize {
    s.find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(s.len())
}
