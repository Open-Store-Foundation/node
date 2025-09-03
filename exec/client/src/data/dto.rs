use crate::data::models::{Artifact, BuildRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DtoPublishing {
    pub id: Option<i64>,
    pub object_address: String,
    pub track_id: i32,
    pub status: i32,
    pub artifact: Artifact,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidPublishingResponse {
    pub published: Vec<DtoPublishing>,
    pub reviewing: Vec<BuildRequest>,
}
