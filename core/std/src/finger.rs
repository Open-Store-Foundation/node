
pub fn sha256(hash: &String) -> String {
    let hash = hash.trim_start_matches("0x");
    
    // Add colons to the fingerprint for better readability
    let formatted_fingerprint = hash
        .to_uppercase()
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|chunk| chunk.iter().collect())
        .collect::<Vec<String>>()
        .join(":");

    formatted_fingerprint
}
