pub fn is_ticker(input: &str) -> bool {
    let trimmed = input.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 6
        && trimmed
            .chars()
            .all(|character| character.is_ascii_alphabetic() || character == '.' || character == '-')
        && trimmed.chars().any(|character| character.is_ascii_alphabetic())
}
