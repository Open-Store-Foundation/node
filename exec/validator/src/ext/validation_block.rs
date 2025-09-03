use openssl::sha::sha256;
use prost::Message;

// TODO v2 clean up
pub fn proto_sha256<T : Message>(block: &T) -> [u8; 32] {
    return sha256(
        block.encode_to_vec()
            .as_slice()
    )
}
