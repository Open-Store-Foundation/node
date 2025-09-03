use std::io;

use bytes::Bytes;
use core_std::endian::get_u32_le;

pub(crate) fn get_len_pref_copy(buf: &mut &[u8]) -> Result<Bytes, io::Error> {
    let result = get_len_pref_slice(buf)?;
    return Ok(Bytes::copy_from_slice(result));
}

pub(crate) fn get_len_pref_slice<'a>(
    source: &mut &'a [u8]
) -> Result<&'a [u8], io::Error> {
    if source.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes in buffer",
        ));
    }

    let len = get_u32_le(source) as usize;
    return get_and_move_slice(source, 4, len);
}

pub(crate) fn get_and_move_slice<'a>(
    source: &mut &'a [u8],
    from: usize,
    size: usize
) -> Result<&'a [u8], io::Error> {
    let to = from + size;
    if source.len() < to {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes in buffer",
        ));
    }

    let result = &source[from..to];
    *source = &source[to..];

    return Ok(result);
}
