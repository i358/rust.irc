pub fn to_hex(t: &str, bytes: &mut String) {
    for c in t.chars() {
        bytes.push_str(&format!("{:02x}", c as u8));
    }
}


pub fn from_hex(hex: &str) -> Result<String, &'static str> {
    if hex.len() % 2 != 0 {
        return Err("Invalid hexadecimal input: length must be even.");
    }

    let mut result = String::new();
    for chunk in hex.as_bytes().chunks(2) {
        let hex_str = std::str::from_utf8(chunk).map_err(|_| "Invalid UTF-8 sequence")?;
        let byte = u8::from_str_radix(hex_str, 16).map_err(|_| "Invalid hexadecimal value")?;
        result.push(byte as char);
    }

    Ok(result)
}
 