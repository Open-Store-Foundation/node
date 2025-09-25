use alloy::primitives::{Address, U256};
use core_std::hexer;

pub trait ToChecksum {
    fn upper_checksum(&self) -> String;
    fn lower_checksum(&self) -> String;
}

impl ToChecksum for Address {
    fn upper_checksum(&self) -> String {
        return hexer::encode_upper_pref(&self)
    }
    fn lower_checksum(&self) -> String { return hexer::encode_lower_pref(&self) }
}

impl ToChecksum for String {
    fn upper_checksum(&self) -> String {
        if !self.starts_with("0x") {
            return self.to_string();
        }

        return format!("0x{}", self[2..].to_uppercase());
    }

    fn lower_checksum(&self) -> String {
        if !self.starts_with("0x") {
            return self.to_string();
        }

        return self.to_lowercase();
    }
}

pub fn write_2bit_status(to: &mut U256, id: usize, status: u32) {
    match status {
        0 => {
            to.set_bit(id, false);
            to.set_bit(id + 1, false);
        }
        1 => {
            to.set_bit(id, true);
            to.set_bit(id + 1, false);
        }
        2 => {
            to.set_bit(id, false);
            to.set_bit(id + 1, true);
        }
        _ => {
            to.set_bit(id, true);
            to.set_bit(id + 1, true);
        }
    }
}

pub fn read_2bit_status(result: U256, i: usize) -> u32 {
    let is_first = result.bit(i);
    let is_second = result.bit(i + 1);

    if !is_first && !is_second {
        return 0; // Unavailable
    } else if !is_first && is_second {
        return 1; // Success
    } else if is_first && !is_second {
        return 2; // -
    } else {
        return 3; // Error
    }
}
