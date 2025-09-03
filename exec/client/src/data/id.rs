use derive_more::Display;
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Display, Debug, PartialEq, PartialOrd, Eq, Clone, Copy, Hash, Deserialize, Serialize)]
pub enum CategoryId {
    Unspecified = 0,
    BooksAndReference = 1,
    Business = 2,
    DeveloperTools = 3,
    Education = 4,
    Entertainment = 5,
    Finance = 6,
    FoodAndDrink = 7,
    GraphicsAndDesign = 8,
    HealthAndFitness = 9,
    Lifestyle = 10,
    MagazinesAndNewspapers = 11,
    Medical = 12,
    Music = 13,
    Navigation = 14,
    News = 15,
    PhotoAndVideo = 16,
    Productivity = 17,
    Shopping = 18,
    SocialNetworking = 19,
    Sports = 20,
    Travel = 21,
    Utilities = 22,
    Weather = 23,

    Action = 101,
    Adventure = 102,
    Arcade = 103,
    Board = 104,
    Card = 105,
    Casino = 106,
    Casual = 107,
    Dice = 108,
    Educational = 109,
    Family = 110,
    GameMusic = 111,
    Puzzle = 112,
    Racing = 113,
    RolePlaying = 114,
    Simulation = 115,
    GameSports = 116,
    Strategy = 117,
    Trivia = 118,
    Word = 119,
}

impl CategoryId {
    pub fn type_id(&self) -> ObjTypeId {
        if self > &CategoryId::Unspecified && self <= &Self::Weather {
            return ObjTypeId::App
        }
        
        if self >= &CategoryId::Action && self <= &Self::Word {
            return ObjTypeId::Game
        }
        
        return ObjTypeId::Unspecified;
    }
}

impl Into<i32> for CategoryId {
    fn into(self) -> i32 {
        self as i32
    } 
}

impl From<u16> for CategoryId {
    fn from(value: u16) -> Self {
        return Self::from(value as i32);
    }
}

impl From<i32> for CategoryId {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::BooksAndReference,
            2 => Self::Business,
            3 => Self::DeveloperTools,
            4 => Self::Education,
            5 => Self::Entertainment,
            6 => Self::Finance,
            7 => Self::FoodAndDrink,
            8 => Self::GraphicsAndDesign,
            9 => Self::HealthAndFitness,
            10 => Self::Lifestyle,
            11 => Self::MagazinesAndNewspapers,
            12 => Self::Medical,
            13 => Self::Music,
            14 => Self::Navigation,
            15 => Self::News,
            16 => Self::PhotoAndVideo,
            17 => Self::Productivity,
            18 => Self::Shopping,
            19 => Self::SocialNetworking,
            20 => Self::Sports,
            21 => Self::Travel,
            22 => Self::Utilities,
            23 => Self::Weather,
            101 => Self::Action,
            102 => Self::Adventure,
            103 => Self::Arcade,
            104 => Self::Board,
            105 => Self::Card,
            106 => Self::Casino,
            107 => Self::Casual,
            108 => Self::Dice,
            109 => Self::Educational,
            110 => Self::Family,
            111 => Self::GameMusic,
            112 => Self::Puzzle,
            113 => Self::Racing,
            114 => Self::RolePlaying,
            115 => Self::Simulation,
            116 => Self::GameSports,
            117 => Self::Strategy,
            118 => Self::Trivia,
            119 => Self::Word,
            _ => Self::Unspecified,
        }
    }
}

#[derive(Debug, Clone, Display, Deserialize, Serialize, Type)]
#[repr(i32)]
pub enum TrackId {
    #[display("unspecified")]
    #[serde(rename = "unspecified")]
    Unspecified = 0,
    #[display("release")]
    #[serde(rename = "release")]
    Release = 1,
    #[display("beta")]
    #[serde(rename = "beta")]
    Beta = 2,
    #[display("alpha")]
    #[serde(rename = "alpha")]
    Alpha = 3,
}

impl Into<i32> for TrackId {
    fn into(self) -> i32 {
        self as i32
    }
}

impl From<i32> for TrackId {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Release,
            2 => Self::Beta,
            3 => Self::Alpha,
            _ => Self::Unspecified,
        }
    }
}

impl From<u8> for TrackId {
    fn from(value: u8) -> Self {
        Self::from(value as i32)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Display, Deserialize, Serialize, Type)]
#[repr(i32)]
pub enum PlatformId {
    #[display("unspecified")]
    #[serde(rename = "unspecified")]
    Unspecified = 0,
    #[display("android")]
    #[serde(rename = "android")]
    Android = 1,
    #[display("ios")]
    #[serde(rename = "ios")]
    Ios = 2,
    #[display("windows")]
    #[serde(rename = "windows")]
    Windows = 3,
    #[display("macos")]
    #[serde(rename = "macos")]
    Macos = 4,
    #[display("linux")]
    #[serde(rename = "linux")]
    Linux = 5,
    #[display("web")]
    #[serde(rename = "web")]
    Web = 6,
    #[display("cli")]
    #[serde(rename = "cli")]
    Cli = 7,
    #[display("all")]
    #[serde(rename = "all")]
    All = 100,
}

impl From<u16> for PlatformId {
    fn from(value: u16) -> Self {
        Self::from(value as i32)
    }
}

impl Into<i32> for PlatformId {
    fn into(self) -> i32 {
        self as i32
    } 
}

impl From<i32> for PlatformId {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Android,
            2 => Self::Ios,
            3 => Self::Windows,
            4 => Self::Macos,
            5 => Self::Linux,
            6 => Self::Web,
            7 => Self::Cli,
            100 => Self::All,
            _ => Self::Unspecified,
        }
    }
}


#[derive(Debug, Clone, Display, Deserialize, Serialize, Type)]
#[repr(i32)]
pub enum ObjTypeId {
    #[display("unspecified")]
    #[serde(rename = "unspecified")]
    Unspecified = 0,
    #[display("app")]
    #[serde(rename = "app")]
    App = 1,
    #[display("game")]
    #[serde(rename = "game")]
    Game = 2,
    #[display("site")]
    #[serde(rename = "site")]
    Site = 3,
}

impl Into<i32> for ObjTypeId {
    fn into(self) -> i32 {
        self as i32
    } 
}

impl From<i32> for ObjTypeId {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::App,
            2 => Self::Game,
            3 => Self::Site,
            _ => Self::Unspecified,
        }
    }
}

#[derive(Debug, Clone, Display, Deserialize, Serialize, Type)]
#[repr(i32)]
pub enum ReqTypeId {
    #[display("unspecified")]
    Unspecified = 0,
    #[display("android_build")]
    AndroidBuild = 1,
}

impl From<i32> for ReqTypeId {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::AndroidBuild,
            _ => Self::Unspecified,
        }
    }
}

impl Into<i32> for ReqTypeId {
    fn into(self) -> i32 {
        self as i32
    }
}

impl From<u8> for ReqTypeId {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::AndroidBuild,
            _ => Self::Unspecified,
        }
    }
}