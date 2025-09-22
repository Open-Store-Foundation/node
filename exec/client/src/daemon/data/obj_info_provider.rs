use crate::data::id::{CategoryId, PlatformId, ReqTypeId, TrackId};
use crate::data::models::{BuildRequest, NewArtifact, NewAsset, NewBuildRequest, Publishing};
use alloy::primitives::Address;
use cloud_gf::client::{GfError, GreenfieldClient};
use codegen_contracts::ext::ToChecksum;
use core_std::hexer;
use net_client::node::result::EthError;
use service_sc::obj::ScObjService;
use std::sync::Arc;
use alloy::hex::ToHexExt;
use chrono::DateTime;
use thiserror::Error;

pub type DaemonResult<T> = Result<T, DaemonError>;

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("Eth error: {0}")]
    Eth(#[from] EthError),
    
    #[error("Gf error: {0}")]
    Gf(#[from] GfError),
}

pub struct DaemonFactory {
    obj_service: Arc<ScObjService>,
    greenfield: Arc<GreenfieldClient>,
}

impl DaemonFactory {

    pub fn new(obj_service: Arc<ScObjService>, greenfield: Arc<GreenfieldClient>) -> Self {
        Self { obj_service, greenfield }
    }
    
    pub fn create_publishing(
        &self,
        obj: Address,
        track_id: TrackId,
        version: i64,
    ) -> Publishing {
        let publishing = Publishing {
            object_address: obj.upper_checksum(),
            track_id,
            version_code: version,
            is_active: true,
        };
        
        return publishing;
    }
    
    pub fn create_build_request(
        &self,
        request_id: u64,
        obj: Address,
        track_id: u8,
        status: Option<i32>,
        version_code: i64,
        owner_version: u64,
        timestamp: Option<u64>
    ) -> NewBuildRequest {
        let time = if let Some(time) = timestamp {
            DateTime::from_timestamp_millis(time as i64)
        } else {
            None
        };

        let build_request = NewBuildRequest {
            id: request_id as i64,
            object_address: obj.upper_checksum(),
            request_type_id: ReqTypeId::AndroidBuild,
            track_id: TrackId::from(track_id),
            status,
            version_code,
            owner_version,
            created_at: time
        };
        
        return build_request;
    }
    
    pub async fn create_obj(&self, obj: Address) -> DaemonResult<NewAsset> {
        let general = self.obj_service.get_general_info(obj)
            .await?;
        
        let bucket_name = self.obj_service.get_owner_name(obj)
            .await?;
        
        let response = self.greenfield.get_object_logo_info(&bucket_name, &general.id)
            .await?;
        
        let category = CategoryId::from(general.categoryId);
        
        let new_obj = NewAsset {
            name: general.name,
            id: general.id,
            address: obj.upper_checksum(),
            logo: response,
            description: Some(general.description),
            type_id: category.type_id(),
            category_id:  category,
            platform_id: PlatformId::from(general.platformId),
            is_os_verified: false,
            is_hidden: false,
            price: 0,
        };
        
        return Ok(new_obj);
    }
    
    pub async fn create_artifact(&self, obj: Address, version: i64) -> DaemonResult<NewArtifact> {
        let build = self.obj_service.get_artifact(obj, version)
            .await?;
        
        let obj_hex = hexer::encode_upper_pref(build.referenceId.as_ref());
        let info = self.greenfield.get_object_meta_by_id(obj_hex.as_str())
            .await?;

        let payload_size = info.object_info.payload_size.parse::<usize>()
            .map_err(|e| DaemonError::Gf(GfError::ResponseFormat))?;
        
        let artifact = NewArtifact {
            object_ref: hexer::encode_upper_pref(build.referenceId.as_ref()),
            object_address: obj.upper_checksum(),
            protocol_id: build.protocolId as i32,
            size: payload_size as i64,
            version_code: build.versionCode.to(),
            version_name: Some(build.versionName),
            checksum: build.checksum.encode_hex_upper_with_prefix(),
        };

        return Ok(artifact);
    }
}
