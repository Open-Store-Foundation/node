use crate::hexer;
use alloy_primitives::Address;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// TODO use this impl later for multi addresses
#[derive(Clone, Copy, Hash, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Display, Debug)]
#[serde(try_from = "String", into = "String")]
pub enum Adresse {
    #[display("{_0}")]
    Alloy(Address)
}

impl Adresse {
    pub fn from_str<S: AsRef<str>>(value: S) -> Result<Self, String> {
        Address::from_str(value.as_ref())
            .map(Adresse::Alloy)
            .map_err(|e| e.to_string())
    }

    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Adresse::Alloy(Address::from(bytes))
    }

    pub fn as_alloy(&self) -> &Address {
        match self {
            Adresse::Alloy(addr) => addr
        }
    }

    pub fn into_alloy(self) -> Address {
        match self {
            Adresse::Alloy(addr) => addr
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Adresse::Alloy(addr) => {
                addr.0 == Address::ZERO.0
            }
        }
    }
}

impl TryFrom<String> for Adresse {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Adresse::from_str(value)
    }
}

impl From<Adresse> for String {
    fn from(value: Adresse) -> Self {
        match value {
            Adresse::Alloy(addr) => hexer::encode_lower_pref(addr)
        }
    }
}

impl From<Address> for Adresse {
    fn from(value: Address) -> Self {
        Adresse::Alloy(value)
    }
}

impl From<Adresse> for Address {
    fn from(value: Adresse) -> Self {
        match value {
            Adresse::Alloy(addr) => addr
        }
    }
}

impl AsRef<[u8]> for Adresse {
    fn as_ref(&self) -> &[u8] {
        match self {
            Adresse::Alloy(addr) => addr.as_ref()
        }
    }
}
