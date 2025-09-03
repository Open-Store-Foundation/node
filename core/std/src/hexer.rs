
pub fn encode_pref<T: AsRef<[u8]>>(data: T) -> String {
    format!("0x{}", hex::encode(data))
}

pub fn encode_upper_pref<T: AsRef<[u8]>>(data: T) -> String {
    format!("0x{}", hex::encode_upper(data))
}
