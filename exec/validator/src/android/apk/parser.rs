use std::fmt::Debug;
use std::io::SeekFrom;
use tokio::fs::File;
use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use core_std::endian::{get_u16_le, get_u32_le, get_u64_le};
use crate::android::apk::reader;
use crate::android::status::ApkValidationStatus;

pub const APK_SIG_BLOCK_MAGIC_HI: u64 = 0x3234206b636f6c42;
pub const APK_SIG_BLOCK_MAGIC_LO: u64 = 0x20676953204b5041;
pub const ZIP_EOCD_SEPARATOR: u32 = 0x07064b50;
pub const APK_SIG_BLOCK_MIN_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct ApkOffsets {
    pub sign: u64,
    pub cd: u64,
    pub eocd: u64,
}

#[derive(Debug, Clone)]
pub struct ApkBlockInfo {
    pub offsets: ApkOffsets,
    pub signers_block: Bytes,
    pub eocd: Bytes,
}

const MIN_EOCD_SIZE: usize = 22;
const MIN_EOCD_SIZE_LONG: u64 = 22;

#[derive(Default)]
pub struct ApkParser;

impl ApkParser {

    pub async fn parse_apk_info(
        &self,
        mut apk: &mut File,
        block_id: u32
    ) -> Result<ApkBlockInfo, ApkValidationStatus> {
        let (eocd_offset, eocd) = self.get_eocd(&mut apk)
            .await?;

        if self.is_zip64_eocd_locator_present(&mut apk, eocd_offset).await? {
            return Err(ApkValidationStatus::Zip64NotSupported);
        }

        let cd_offset = self.get_central_dir_offset(&eocd, eocd_offset)?;

        let (sign_offset, apk_block) = self.find_apk_sign_block(&mut apk, cd_offset)
                .await?;

        let signers_block = self.find_apk_sig_block(&apk_block, block_id)
                .await?;

        Ok(
            ApkBlockInfo {
                offsets: ApkOffsets {
                    sign: sign_offset,
                    cd: cd_offset,
                    eocd: eocd_offset,
                },
                signers_block,
                eocd,
            }
        )
    }

    fn get_central_dir_offset(
        &self,
        eocd: &[u8],
        eocd_offset: u64
    ) -> Result<u64, ApkValidationStatus> {
        if eocd.len() < 20 {
            return Err(ApkValidationStatus::InvalidApkFormat);       
        }
        
        let central_dir_offset = get_u32_le(&eocd[16..20]) as u64;
        if central_dir_offset > eocd_offset {
            return Err(ApkValidationStatus::InvalidApkFormat);
        }

        let central_dir_size = get_u32_le(&eocd[12..16]) as u64;
        if central_dir_offset + central_dir_size != eocd_offset {
            return Err(ApkValidationStatus::InvalidApkFormat);
        }

        Ok(central_dir_offset)
    }

    pub async fn get_eocd(
        &self,
        apk: &mut File
    ) -> Result<(u64, Bytes), ApkValidationStatus> {
        let file_size = apk.seek(SeekFrom::End(0))
            .await
            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

        if file_size < 22 {
            return Err(ApkValidationStatus::SignaturesNotFound);
        }

        let max_eocd_size = u16::MAX as u64 + MIN_EOCD_SIZE_LONG;
        let (start, end) = if file_size < max_eocd_size {
            (0, file_size - MIN_EOCD_SIZE_LONG)
        } else {
            (file_size - max_eocd_size, file_size - MIN_EOCD_SIZE_LONG)
        };

        let mut eocd_buf = [0u8; MIN_EOCD_SIZE];
        for i in (start..=end).rev() {
            apk.seek(SeekFrom::Start(i))
                .await
                .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

            apk.read_exact(&mut eocd_buf)
                .await
                .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

            if get_u32_le(&eocd_buf) == 0x06054b50 {
                let comment_len = get_u16_le(&eocd_buf[20..MIN_EOCD_SIZE]) as usize;
                let expected_eocd_size = MIN_EOCD_SIZE + comment_len;

                let result = if expected_eocd_size <= MIN_EOCD_SIZE {
                    Bytes::copy_from_slice(&eocd_buf)
                } else {
                    let mut full_eocd_buf = vec![0u8; expected_eocd_size];

                    apk.seek(SeekFrom::Start(i))
                        .await
                        .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

                    apk.read_exact(&mut full_eocd_buf)
                        .await
                        .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

                    Bytes::from(full_eocd_buf)
                };

                return Ok((i, result))
            }
        }

        Err(ApkValidationStatus::InvalidApkFormat)
    }

    async fn find_apk_sign_block(
        &self,
        apk: &mut File,
        central_dir_offset: u64,
    ) -> Result<(u64, Bytes), ApkValidationStatus> {
        if central_dir_offset < APK_SIG_BLOCK_MIN_SIZE as u64 {
            return Err(ApkValidationStatus::InvalidApkFormat);
        }

        apk.seek(SeekFrom::Start(central_dir_offset - 24))
            .await
            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

        let mut footer = [0u8; 24];
        apk.read_exact(&mut footer)
            .await
            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

        if get_u64_le(&footer[8..16]) != APK_SIG_BLOCK_MAGIC_LO
            || get_u64_le(&footer[16..24]) != APK_SIG_BLOCK_MAGIC_HI
        {
            return Err(ApkValidationStatus::SignaturesNotFound);
        }

        let apk_sig_block_size_in_footer = get_u64_le(&footer[0..8]);
        if apk_sig_block_size_in_footer < 24 || apk_sig_block_size_in_footer > u32::MAX as u64 - 8 { // 8 len
            return Err(ApkValidationStatus::InvalidApkFormat);
        }

        let total_size = (apk_sig_block_size_in_footer + 8) as usize;
        if central_dir_offset < total_size as u64 {
            return Err(ApkValidationStatus::InvalidApkFormat);
        }

        let apk_sig_block_offset = central_dir_offset - total_size as u64;
        apk.seek(SeekFrom::Start(apk_sig_block_offset))
            .await
            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

        let mut apk_sig_block = vec![0u8; total_size];

        apk.read_exact(&mut apk_sig_block)
            .await
            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

        let apk_sig_block_size_in_header = get_u64_le(&apk_sig_block);
        if apk_sig_block_size_in_header != apk_sig_block_size_in_footer {
            return Err(ApkValidationStatus::InvalidApkFormat);
        }

        Ok((apk_sig_block_offset, Bytes::from(apk_sig_block)))
    }

    async fn find_apk_sig_block(
        &self,
        apk_signing_block: &[u8],
        block_id: u32,
    ) -> Result<Bytes, ApkValidationStatus> {
        if apk_signing_block.len() < 32 {
            return Err(ApkValidationStatus::InvalidApkFormat);
        }

        let mut pairs = &apk_signing_block[8..apk_signing_block.len() - 24];
        while !pairs.is_empty() {
            if pairs.len() < 8 {
                return Err(ApkValidationStatus::InvalidApkFormat);
            }

            let len = get_u64_le(&pairs[0..8]) as usize;
            let offset = len + 8;
            if offset < 12 || offset > pairs.len() {
                return Err(ApkValidationStatus::InvalidApkFormat);
            }

            let id = get_u32_le(&pairs[8..12]);
            if id == block_id {
                let buff = reader::get_and_move_slice(&mut pairs, 12, len - 4)
                    .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

                return Ok(Bytes::copy_from_slice(buff));
            }

            pairs = &pairs[offset..];
        }

        Err(ApkValidationStatus::SignaturesNotFound)
    }

    async fn is_zip64_eocd_locator_present(
        &self,
        apk: &mut File,
        eocd_offset: u64,
    ) -> Result<bool, ApkValidationStatus> {
        if eocd_offset < 20 {
            return Ok(false);
        }

        let locator_offset = eocd_offset - 20;

        apk.seek(SeekFrom::Start(locator_offset))
            .await
            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

        let data = apk.read_u32_le()
            .await
            .map_err(|_| ApkValidationStatus::InvalidApkFormat)?;

        Ok(data == ZIP_EOCD_SEPARATOR)
    }
}
