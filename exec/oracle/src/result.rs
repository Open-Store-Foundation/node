use derive_more::Display;

#[derive(Debug, Display)]
pub enum AssetlinkError {
    CantDecodeEvent,
    CantFinalize,
}

pub type AssetlinkResult<T> = Result<T, AssetlinkError>;
