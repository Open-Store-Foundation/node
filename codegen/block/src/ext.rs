use crate::block::{ValidationBlock, ValidationResult};
use crate::status::ApkValidationStatus;
use crate::FileHashAlgo;

impl ValidationResult {
    pub fn default_with(request_id: u64, request_type: u8, target: String) -> ValidationResult {
        let mut result = ValidationResult::unavailable(request_id);
        result.asset_address = target;
        result.request_type = request_type as u32;
        result
    }
    
    pub fn unavailable(request_id: u64) -> ValidationResult {
        ValidationResult {
            request_id,
            request_type: 0,
            asset_address: "0x".into(),
            artifact_ref_id: "0x".into(),
            artifact_protocol: 0,
            object_version: 0,
            owner_version: 0,
            track_id: 0,
            file_hash_algorithm: FileHashAlgo::None.code(),
            file_hash: "0x".into(),
            proofs: None,
            status: ApkValidationStatus::Unavailable.code(),
        }
    }
}

impl ValidationBlock {
    pub fn from_request_id(&self) -> Option<u64> {
        return self.requests.first().map_or(None, |r| Some(r.request_id));
    }
    
    pub fn to_request_id(&self) -> Option<u64> {
        return self.requests.last().map_or(None, |r| Some(r.request_id + 1));
    }
}