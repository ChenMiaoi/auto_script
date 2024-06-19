pub fn get_filed(line: &str, skipped: &str) -> String {
    line[skipped.len()..].trim().to_string()
}
