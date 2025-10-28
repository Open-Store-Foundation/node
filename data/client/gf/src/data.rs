use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VirtualGroupsFamily {
    pub global_virtual_group_family: GroupsFamily,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroupsFamily {
    pub id: i32,
    pub primary_sp_id: i32,
    pub global_virtual_group_ids: Vec<i32>,
}

//////

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpProviders {
    pub sps: Vec<SpProvider>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpProvider {
    pub id: i32,
    pub operator_address: String,
    pub endpoint: String,
    pub status: String,
}

//////

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BucketMetaHead {
    pub bucket_info: BucketInfo,
    pub extra_info: BucketExtraInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BucketExtraInfo {
    pub is_rate_limited: bool,
    pub flow_rate_limit: String,
    pub current_flow_rate: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BucketInfo {
    pub id: String,
    pub owner: String,
    pub source_type: String,
    pub charged_read_quota: String,
    pub bucket_status: String,
    pub global_virtual_group_family_id: i32,
}

///////

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeadObjectMeta {
    pub object_info: ObjectInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectInfo {
    pub id: String,
    pub owner: String,
    pub creator: String,
    pub bucket_name: String,
    pub object_name: String,
    pub payload_size: String,
    pub visibility: VisibilityType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VisibilityType {
    VisibilityTypeUnspecified,
    VisibilityTypePublicRead,
    VisibilityTypePrivate,
    VisibilityTypeInherit,
    Unrecognized,
}

