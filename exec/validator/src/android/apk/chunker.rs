use std::cmp::min;
use std::collections::HashMap;
use std::io::{Cursor, SeekFrom};
use std::ops::Range;

use bytes::Bytes;
use ring::digest::Digest;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, BufReader};
use core_log::init_tracer;
use core_std::endian::put_u32_le;
use crate::android::apk::crypto::ApkChunkDigest;
use crate::android::apk::parser::ApkBlockInfo;
use crate::android::status::{ApkValidationStatus, ApkResult};

pub const CHUNK_SIZE_BYTES: usize = 1024 * 1024;
const ROOT_HASH_SIZE: usize = 32;
const SOURCE_LENGTH_SIZE: usize = 8;

trait AsyncReadSeek: AsyncRead + AsyncSeek {}
impl AsyncReadSeek for BufReader<&mut File> {}
impl AsyncReadSeek for BufReader<Cursor<Vec<u8>>> {}

struct RangedBuffer<'a> {
    buffer: Box<dyn AsyncReadSeek + Unpin + Send + 'a>,
    ranges: Vec<Range<usize>>,
}

impl <'a> RangedBuffer<'a> {
    pub fn new(buffer: Box<dyn AsyncReadSeek + Unpin + Send + 'a>, ranges: Vec<Range<usize>>) -> Self {
        Self { buffer, ranges }
    }
}

#[derive(Default)]
pub struct ApkChunker;

impl ApkChunker {

    pub async fn verify_digest(
        &self,
        apk: &mut File,
        expected_digests: &HashMap<ApkChunkDigest, Bytes>,
        block_info: &ApkBlockInfo,
    ) -> Result<(), ApkValidationStatus> {
        if expected_digests.is_empty() {
            return Err(ApkValidationStatus::NoDigestFound);
        }

        let mut never_verified = true;
        let mut digests_algos = HashMap::new();

        if let Some(digest) = expected_digests.get(&ApkChunkDigest::Sha256) {
            digests_algos.insert(ApkChunkDigest::Sha256, digest.clone());
        }

        if let Some(digest) = expected_digests.get(&ApkChunkDigest::Sha512) {
            digests_algos.insert(ApkChunkDigest::Sha512, digest.clone());
        }

        if !digests_algos.is_empty() {
            self.verify_integrity_for_1mb_chunk_based_algorithm(&digests_algos, apk, block_info)
                .await?;
            never_verified = false;
        }

        if never_verified {
            return Err(ApkValidationStatus::NoKnownDigestToCheck);
        }

        Ok(())
    }

    async fn verify_integrity_for_1mb_chunk_based_algorithm(
        &self,
        expected_digests: &HashMap<ApkChunkDigest, Bytes>,
        apk: &mut File,
        block_info: &ApkBlockInfo,
    ) -> Result<(), ApkValidationStatus> {
        let digest_algorithms: Vec<ApkChunkDigest> = expected_digests.keys().cloned().collect();

        let actual_digests = self
            .compute_digest_from(&digest_algorithms, apk, block_info)
            .await?;

        for (algo, actual_digest) in actual_digests {
            let expected_digest = expected_digests.get(&algo)
                .ok_or(ApkValidationStatus::DigestAlgorithmNotFound)?;

            if expected_digest != actual_digest.as_ref() {
                return Err(ApkValidationStatus::DigestMismatch);
            }
        }

        Ok(())
    }

    async fn compute_digest_from(
        &self,
        digest_algorithms: &Vec<ApkChunkDigest>,
        apk: &mut File,
        block_info: &ApkBlockInfo,
    ) -> ApkResult<HashMap<ApkChunkDigest, Digest>> {
        let offsets = &block_info.offsets;

        let buffer = RangedBuffer::new(
            Box::new(BufReader::new(apk)),
            vec![
                0..offsets.sign as usize,
                offsets.cd as usize..offsets.eocd as usize,
            ],
        );

        let mut eocd_buf = block_info.eocd.to_vec();
        let eocd_size = eocd_buf.len();
        put_u32_le(&mut eocd_buf[16..20], offsets.sign as u32);

        let eocd_buffer = RangedBuffer::new(
            Box::new(BufReader::new(Cursor::new(eocd_buf))),
            vec![0..eocd_size],
        );

        return self.compute_digests(digest_algorithms, vec![buffer, eocd_buffer])
            .await;
    }

    async fn compute_digests(
        &self,
        digest_algorithms: &Vec<ApkChunkDigest>,
        contents: Vec<RangedBuffer<'_>>,
    ) -> ApkResult<HashMap<ApkChunkDigest, Digest>> {
        let total_chunk_count = contents
            .iter()
            .flat_map(|input| {
                input.ranges.iter().map(|range| Self::get_chunk_count(range.len()))
            })
            .sum::<usize>();

        if total_chunk_count > (i32::MAX / 1024) as usize {
            return Err(ApkValidationStatus::TooManyChunks);
        }

        let mut digest_buffers = HashMap::new();
        for algo in digest_algorithms {
            let size = algo.digest_size();
            let mut digests_buffer = vec![0; 5 + total_chunk_count * size];

            digests_buffer[0] = 0x5a;
            put_u32_le(&mut digests_buffer[1..5], total_chunk_count as u32);

            digest_buffers.insert(algo.clone(), digests_buffer);
        }

        let mut chunk_content_prefix = [0u8; 5];
        chunk_content_prefix[0] = 0xa5;

        let mut chunk_content = vec![0u8; CHUNK_SIZE_BYTES];
        let mut chunk_count = 0;

        for mut input in contents {
            for range in input.ranges {
                input.buffer.seek(SeekFrom::Start(range.start as u64))
                    .await
                    .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

                let mut input_remaining = range.len();

                while input_remaining > 0 {
                    let chunk_size = min(input_remaining, CHUNK_SIZE_BYTES);
                    put_u32_le(&mut chunk_content_prefix[1..5], chunk_size as u32);

                    for (algo, data) in &mut digest_buffers {
                        let size = input.buffer.read_exact(&mut chunk_content[0..chunk_size])
                            .await
                            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

                        if size != chunk_size {
                            return Err(ApkValidationStatus::InvalidApkFormat);
                        }

                        let mut context = algo.context();
                        context.update(&chunk_content_prefix);
                        context.update(&chunk_content[0..size]);

                        let size = algo.digest_size();
                        let start_index = 5 + chunk_count * size;
                        let end_index = start_index + size;

                        data[start_index..end_index].copy_from_slice(context.finish().as_ref());
                    }

                    input_remaining -= chunk_size;
                    chunk_count += 1;
                }
            }
        }

        return Ok(
            digest_buffers
                .into_iter()
                .map(|(algo, data)| {
                    let mut context = algo.context();
                    context.update(data.as_slice());
                    let result = context.finish();
                    (algo, result)
                })
                .collect()
        );
    }

    fn get_chunk_count(input_size_bytes: usize) -> usize {
        (input_size_bytes + CHUNK_SIZE_BYTES - 1) / CHUNK_SIZE_BYTES
    }
}

#[tokio::test]
async fn test() {
    init_tracer();
    let checker = ApkChunker::default();
    assert!(true); // Add meaningful test cases
}
