use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use codegen_stat::stat::{event_wrapper, EventWrapper, ObjTypeId, ObjectInstalled, ObjectUpdated, PlatformId};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::{env, error::Error, time::Duration};
use derive_more::Display;
use tracing::{error, info, instrument, warn};

#[derive(Row, Serialize, Deserialize, Debug, Clone)]
pub struct ObjectEvent {
    event_name: String,

    object_id: i64,

    category_id: i32,
    platform_id: i32,
    obj_type_id: i32,

    artifact_id: String,
    artifact_protocol: i32,

    version_code: Option<i32>,
    version_name: Option<String>,

    to_version_code: Option<i32>,
    to_version_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Display, Clone)]
pub enum EventType {
    #[display("ObjectDownloaded")]
    Downloaded,

    #[display("ObjectInstalled")]
    Installed,

    #[display("ObjectUpdated")]
    Updated,

    #[display("ObjectDeleted")]
    Deleted,
}

impl TryFrom<EventWrapper> for ObjectEvent {

    type Error = ();

    fn try_from(event_wrapper: EventWrapper) -> Result<ObjectEvent, ()> {
        match event_wrapper.event_payload {
            Some(event_wrapper::EventPayload::Downloaded(event)) => {
                info!(
                    object_id = event.object_id,
                    event_type = "downloaded",
                    "Mapping Downloaded event"
                );

                let Ok(platform_id) = PlatformId::try_from(event.platform_id) else {
                    return Err(());
                };

                let Ok(type_id) = ObjTypeId::try_from(event.obj_type_id) else {
                    return Err(());
                };

                Ok(
                    ObjectEvent {
                        event_name: EventType::Installed.to_string(),

                        object_id: event.object_id,
                        obj_type_id: type_id.into(),
                        platform_id: platform_id.into(),
                        category_id: event.category_id,

                        artifact_id: event.artifact_id,
                        artifact_protocol: event.artifact_protocol,

                        version_code: Some(event.version_code),
                        version_name: Some(event.version_name),

                        to_version_code: None,
                        to_version_name: None,
                    }
                )
            }
            Some(event_wrapper::EventPayload::Installed(event)) => {
                info!(
                    object_id = event.object_id,
                    event_type = "installed",
                    "Mapping Installed event"
                );

                let Ok(platform_id) = PlatformId::try_from(event.platform_id) else {
                    return Err(());
                };

                let Ok(type_id) = ObjTypeId::try_from(event.obj_type_id) else {
                    return Err(());
                };

                Ok(
                    ObjectEvent {
                        event_name: EventType::Installed.to_string(),

                        object_id: event.object_id,
                        obj_type_id: type_id.into(),
                        platform_id: platform_id.into(),
                        category_id: event.category_id,

                        artifact_id: event.artifact_id,
                        artifact_protocol: event.artifact_protocol,

                        version_code: Some(event.version_code),
                        version_name: Some(event.version_name),

                        to_version_code: None,
                        to_version_name: None,
                    }
                )
            }

            Some(event_wrapper::EventPayload::Updated(event)) => {
                info!(
                    object_id = event.object_id,
                    event_type = "updated",
                    "Mapping Updated event"
                );

                let Ok(platform_id) = PlatformId::try_from(event.platform_id) else {
                    return Err(());
                };

                let Ok(type_id) = ObjTypeId::try_from(event.obj_type_id) else {
                    return Err(());
                };

                Ok(
                    ObjectEvent {
                        event_name: EventType::Updated.to_string(),

                        object_id: event.object_id,
                        obj_type_id: type_id.into(),
                        platform_id: platform_id.into(),
                        category_id: event.category_id,

                        artifact_id: event.artifact_id,
                        artifact_protocol: event.artifact_protocol,

                        version_code: Some(event.version_code),
                        version_name: Some(event.version_name),

                        to_version_code: Some(event.to_version_code),
                        to_version_name: Some(event.to_version_name),
                    }
                )
            }

            Some(event_wrapper::EventPayload::Deleted(event)) => {
                info!(
                    object_id = event.object_id,
                    event_type = "deleted",
                    "Mapping Deleted event"
                );

                let Ok(platform_id) = PlatformId::try_from(event.platform_id) else {
                    return Err(());
                };

                let Ok(type_id) = ObjTypeId::try_from(event.obj_type_id) else {
                    return Err(());
                };

                Ok(
                    ObjectEvent {
                        event_name: EventType::Deleted.to_string(),

                        object_id: event.object_id,
                        obj_type_id: type_id.into(),
                        platform_id: platform_id.into(),
                        category_id: event.category_id,

                        artifact_id: event.artifact_id,
                        artifact_protocol: event.artifact_protocol,

                        version_code: Some(event.version_code),
                        version_name: Some(event.version_name),

                        to_version_code: None,
                        to_version_name: None,
                    }
                )
            }
            None => {
                Err(())
            }
        }
    }
}

// #[tokio::test]
// async fn test_data() -> Vec<Bytes> {
//     // In reality, this comes from your Kafka consumer loop
//     let mut batch = Vec::new();
//
//     // Example: Create one of each event type
//     let installed = ObjectInstalled {
//         product_id: ProductId::Android as i32,
//         object_id: 101,
//         object_type: ObjectType::Application as i32,
//         content_id: "com.example.app".to_string(),
//         version_code: 10,
//         version_name: "1.0.0".to_string(),
//     };
//     let updated = ObjectUpdated {
//         product_id: ProductId::Ios as i32,
//         object_id: 202,
//         object_type: ObjectType::Game as i32,
//         content_id: "ios.game.id".to_string(),
//         version_code: 5,                 // "from" version code
//         version_name: "0.9".to_string(), // "from" version name
//         to_version_code: 6,
//         to_version_name: "100".to_string(), // i32 according to proto!
//     };
//     let deleted = ObjectDeleted {
//         product_id: ProductId::Web as i32,
//         object_id: 303,
//         object_type: ObjectType::Site as i32,
//         content_id: "example.com".to_string(),
//         version_code: 1,
//         version_name: "live".to_string(),
//     };
//
//     let wrapper_installed = EventWrapper {
//         event_payload: Some(event_wrapper::EventPayload::Installed(installed)),
//     };
//     let wrapper_updated = EventWrapper {
//         event_payload: Some(event_wrapper::EventPayload::Updated(updated)),
//     };
//     let wrapper_deleted = EventWrapper {
//         event_payload: Some(event_wrapper::EventPayload::Deleted(deleted)),
//     };
//
//     batch.push(Bytes::from(wrapper_installed.encode_to_vec()));
//     batch.push(Bytes::from(wrapper_updated.encode_to_vec()));
//     batch.push(Bytes::from(wrapper_deleted.encode_to_vec()));
//
//     batch
// }
