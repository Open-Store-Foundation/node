
pub mod block;
pub mod status;
pub mod ext;

pub enum FileHashAlgo {
    None,
    Blake3,
    Sha256,
}

impl FileHashAlgo {
    pub fn code(&self) -> String {
        match self {
            FileHashAlgo::None => "none".into(),
            FileHashAlgo::Blake3 => "blake3".into(),
            FileHashAlgo::Sha256 => "sha256".into(),
        }
    }
}
