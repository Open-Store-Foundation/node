use crate::data::id::{CategoryId, ObjTypeId, PlatformId, ReqTypeId, TrackId};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::hash::{Hash, Hasher};
use chrono::DateTime;

#[derive(Debug, Clone, Hash, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    pub address: String,
}

#[derive(Debug, Clone, Hash, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i32, 
    pub name: String,
    pub type_id: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCategory {
    pub id: i32,
    pub name: String,
    pub type_id: i32,
}

#[derive(Debug, Clone, Hash, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAuthor {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichObject {
    pub id: i64,
    pub name: String,
    pub package_name: String,
    pub address: String,

    pub website: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,

    pub category_id: i32,
    pub platform_id: i32,
    pub type_id: i32,

    pub is_ownership_verified: bool,
    pub is_build_verified: bool,
    pub is_os_verified: bool,

    pub rating: f32,
    pub price: i64,
    pub downloads: i64,
}

#[derive(Debug, Clone, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub package_name: String,
    pub address: String,

    pub website: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,

    pub category_id: i32,
    pub platform_id: i32,
    pub type_id: i32,

    pub is_os_verified: bool,
    pub is_hidden: bool,

    pub rating: f32,
    pub price: i64,
    pub downloads: i64,
}

impl Hash for Asset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.package_name.hash(state);
        self.address.hash(state);
        self.website.hash(state);
        self.logo.hash(state);
        self.description.hash(state);
        self.category_id.hash(state);
        self.platform_id.hash(state);
        self.type_id.hash(state);
        self.is_os_verified.hash(state);
        self.is_hidden.hash(state);

        // Hash the bits of the f32
        self.rating.to_bits().hash(state);

        self.price.hash(state);
        self.downloads.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAsset {
    pub name: String,
    pub id: String,
    pub address: String,

    pub logo: Option<String>,
    pub description: Option<String>,

    pub type_id: ObjTypeId,
    pub category_id: CategoryId,
    pub platform_id: PlatformId,

    pub is_os_verified: bool,
    pub is_hidden: bool,

    pub price: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct AssetlinkSync {
    pub object_address: String,
    pub domain: String,
    pub owner_version: u64,
    pub status: u32,
}

#[derive(Debug, Clone)]
pub struct Publishing {
    pub object_address: String,
    pub track_id: TrackId,
    pub version_code: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct NewBuildRequest {
    pub id: i64,
    pub request_type_id: ReqTypeId,
    pub object_address: String,
    pub track_id: TrackId,
    pub status: Option<i32>,
    pub version_code: i64,
    pub owner_version: u64,
    pub created_at: Option<DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BuildRequest {
    pub id: i64,
    pub request_type_id: ReqTypeId,
    pub object_address: String,
    pub track_id: TrackId,
    pub status: Option<i32>,
    pub version_code: i64,
    pub owner_version: u64,
}

#[derive(Debug, Clone, Hash, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub id: i64, 
    pub object_id: i64,
    pub user_id: String,
    pub rating: i32, 
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReview {
    pub object_id: i64,
    pub user_id: String,
    pub rating: i32,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub id: i64,
    pub object_address: String,
    pub category_id: i32,
    pub subcategory_id: i32,
    pub email: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReport {
    pub object_address: String,
    pub category_id: i32,
    pub subcategory_id: i32,
    pub email: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub id: i64,
    pub ref_id: String,
    pub object_address: String,
    pub protocol_id: i32,
    pub size: i64,
    pub version_name: Option<String>,
    pub version_code: i64,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewArtifact {
    pub object_ref: String,
    pub object_address: String,
    pub protocol_id: i32,
    pub size: i64,
    pub version_name: Option<String>,
    pub version_code: i64,
    pub checksum: String,
}

// FUTURE

#[derive(Debug, Clone, Hash, PartialEq, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: i32,
    pub name: String,
    pub value: Option<String>,
    pub object_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAchievement {
    pub id: i32,
    pub name: String,
    pub value: Option<String>,
    pub object_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Feed {
    sections: Vec<Section>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum Section {
    #[serde(rename = "banner")]
    Banner { objects: Vec<Asset> },

    #[serde(rename = "h_list")]
    HList { objects: Vec<Asset> },

    #[serde(rename = "v_list")]
    VList { objects: Vec<Asset> },

    #[serde(rename = "highlight")]
    Highlight { target: Asset },
}
