use prost::Message;

//==================================================================
// Enums
//==================================================================

/// VisibilityType is the resources public status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum VisibilityType {
    Unspecified = 0,
    PublicRead = 1,
    Private = 2,
    /// If the bucket Visibility is inherit, it's finally set to private. If the object
    /// Visibility is inherit, it's the same as bucket.
    Inherit = 3,
}

/// RedundancyType represents the redundancy algorithm type for object data,
/// which can be either multi-replica or erasure coding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RedundancyType {
    EcType = 0,
    ReplicaType = 1,
}

/// ObjectStatus represents the creation status of an object. After a user successfully
/// sends a CreateObject transaction onto the chain, the status is set to 'Created'.
/// After the Primary Service Provider successfully sends a Seal Object transaction onto
/// the chain, the status is set to 'Sealed'. When a Discontinue Object transaction is
/// received on chain, the status is set to 'Discontinued'.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ObjectStatus {
    Created = 0,
    Sealed = 1,
    Discontinued = 2,
}

/// SourceType represents the source of resource creation, which can
/// from Greenfield native or from a cross-chain transfer from BSC
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SourceType {
    Origin = 0,
    MirrorPending = 1,
    BscCrossChain = 2,
    OpCrossChain = 3,
}

//==================================================================
// Message Structs
//==================================================================

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceTagsTag {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceTags {
    /// tags defines a list of tags the resource has
    #[prost(message, repeated, tag = "1")]
    pub tags: ::prost::alloc::vec::Vec<ResourceTagsTag>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectInfo {
    /// owner is the object owner
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    /// creator is the address of the uploader, it always be same as owner address
    #[prost(string, tag = "2")]
    pub creator: ::prost::alloc::string::String,
    /// bucket_name is the name of the bucket
    #[prost(string, tag = "3")]
    pub bucket_name: ::prost::alloc::string::String,
    /// object_name is the name of object
    #[prost(string, tag = "4")]
    pub object_name: ::prost::alloc::string::String,
    /// id is the unique identifier of object
    #[prost(string, tag = "5")]
    pub id: ::prost::alloc::string::String,
    #[prost(uint32, tag = "6")]
    pub local_virtual_group_id: u32,
    /// payloadSize is the total size of the object payload
    #[prost(uint64, tag = "7")]
    pub payload_size: u64,
    /// visibility defines the highest permissions for object. When an object is public,
    /// everyone can access it.
    #[prost(enumeration = "VisibilityType", tag = "8")]
    pub visibility: i32,
    /// content_type define the format of the object which should be a standard MIME type.
    #[prost(string, tag = "9")]
    pub content_type: ::prost::alloc::string::String,
    /// create_at define the block timestamp when the object is created
    #[prost(int64, tag = "10")]
    pub create_at: i64,
    /// object_status define the upload status of the object.
    #[prost(enumeration = "ObjectStatus", tag = "11")]
    pub object_status: i32,
    /// redundancy_type define the type of the redundancy which can be multi-replication or
    /// EC.
    #[prost(enumeration = "RedundancyType", tag = "12")]
    pub redundancy_type: i32,
    /// source_type define the source of the object.
    #[prost(enumeration = "SourceType", tag = "13")]
    pub source_type: i32,
    /// checksums define the root hash of the pieces which stored in a SP.
    /// add omit tag to omit the field when converting to NFT metadata
    #[prost(bytes = "vec", repeated, tag = "14")]
    pub checksums: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// tags defines a list of tags the object has
    #[prost(message, optional, tag = "15")]
    pub tags: ::core::option::Option<ResourceTags>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHeadObjectRequest {
    #[prost(string, tag = "1")]
    pub bucket_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub object_name: ::prost::alloc::string::String,
}

/// A global virtual group consists of one primary SP (SP) and multiple secondary SP.
/// Every global virtual group must belong to a GVG family, and the objects of each
/// bucket must be stored in a GVG within a group family.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GlobalVirtualGroup {
    /// ID represents the unique identifier of the global virtual group.
    #[prost(uint32, tag = "1")]
    pub id: u32,
    /// Family ID represents the identifier of the GVG family that the group belongs to.
    #[prost(uint32, tag = "2")]
    pub family_id: u32,
    /// Primary SP ID represents the unique identifier of the primary storage provider in
    /// the group.
    #[prost(uint32, tag = "3")]
    pub primary_sp_id: u32,
    /// Secondary SP IDs represents the list of unique identifiers of the secondary storage
    /// providers in the group.
    #[prost(uint32, repeated, tag = "4")]
    pub secondary_sp_ids: ::prost::alloc::vec::Vec<u32>,
    /// Stored size represents the size of the stored objects within the group.
    #[prost(uint64, tag = "5")]
    pub stored_size: u64,
    /// Virtual payment address represents the payment address associated with the group.
    #[prost(string, tag = "6")]
    pub virtual_payment_address: ::prost::alloc::string::String,
    /// Total deposit represents the number of tokens deposited by this storage provider for
    /// staking.
    #[prost(string, tag = "7")]
    pub total_deposit: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHeadObjectResponse {
    #[prost(message, optional, tag = "1")]
    pub object_info: ::core::option::Option<ObjectInfo>,
    #[prost(message, optional, tag = "2")]
    pub global_virtual_group: ::core::option::Option<GlobalVirtualGroup>,
}