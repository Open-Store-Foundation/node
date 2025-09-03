use openssl::sha::{Sha256, Sha512};

pub trait Hasher {
    type Hash : Copy + PartialEq + Into<Vec<u8>> + TryFrom<Vec<u8>> + AsRef<[u8]>;

    fn hash(bufs: &[u8]) -> Self::Hash;
    fn concat_and_hash(left: &[u8], right: &[u8]) -> Self::Hash;
}

#[derive(Default)]
pub struct HasherSha256;

impl Hasher for HasherSha256 {

    type Hash = [u8; 32];

    fn hash(bufs: &[u8]) -> Self::Hash {
        let mut sha = Sha256::new();
        sha.update(bufs);
        sha.finish()
    }

    fn concat_and_hash(left: &[u8], right: &[u8]) -> Self::Hash {
        let mut sha = Sha256::new();
        sha.update(left);
        sha.update(right);
        sha.finish()
    }
}


#[derive(Default)]
pub struct HasherSha512;

impl Hasher for HasherSha512 {

    type Hash = [u8; 64];

    fn hash(bufs: &[u8]) -> Self::Hash {
        let mut sha = Sha512::new();
        sha.update(bufs);
        sha.finish()
    }

    fn concat_and_hash(left: &[u8], right: &[u8]) -> Self::Hash {
        let mut sha = Sha512::new();
        sha.update(left);
        sha.update(right);
        sha.finish()
    }
}
