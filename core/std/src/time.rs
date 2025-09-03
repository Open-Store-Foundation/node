use core::fmt;

use chrono::{DateTime, TimeZone, Utc};

pub fn current_time() -> DateTime<Utc> {
    return Utc::now();
}

pub trait Iso8601 {
    fn to_iso8601(&self) -> String;
}

impl <T : TimeZone> Iso8601 for DateTime<T>
    where
        T::Offset: fmt::Display {

    fn to_iso8601(&self) -> String {
        return self.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    }
}