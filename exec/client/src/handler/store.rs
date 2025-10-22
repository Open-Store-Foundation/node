use crate::data::id::{ObjTypeId, PlatformId};
use crate::env::default_page_size;
use crate::result::{ClientError, ClientResult};
use crate::state::ClientState;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use headers::{HeaderMapExt, IfNoneMatch};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize, Debug)]
pub struct FeedParams {
    #[serde(rename = "type")]
    pub type_id: ObjTypeId,
    #[serde(rename = "platform")]
    pub platform: PlatformId,
}

#[derive(Deserialize, Debug)]
pub struct ChartParams {
    #[serde(default = "default_page_size")]
    pub size: i64,
    #[serde(default)]
    pub offset: i64,
    pub category_id: Option<i32>,
    #[serde(rename = "type")]
    pub type_id: Option<ObjTypeId>,
    pub platform: PlatformId,
}

fn get_feed_key(params: &FeedParams) -> String {
    let type_id: i32 = params.type_id.clone().into();
    return format!("static:get_feed:{}", type_id);
}

fn get_feed_etag_key(params: &FeedParams) -> String {
    let type_id: i32 = params.type_id.clone().into(); 
    return format!("cache:etag:get_feed:{}", type_id);
}

fn get_feed_ttl() -> Option<u64> {
    return Some(60 * 60);
}

pub async fn get_feed(
    State(state): State<ClientState>,
    Query(params): Query<FeedParams>,
    headers: HeaderMap,
) -> ClientResult<Response> {
    let etag_key = get_feed_etag_key(&params);
    let user_etag = headers.typed_get::<IfNoneMatch>();

    return state.etag_handler.etag_cache_or_static(
        etag_key,
        user_etag,
        get_feed_key(&params)
    ).await
}

fn get_categories_etag() -> String {
    return "cache:etag:get_categories".to_string();
}

fn get_categories_ttl() -> Option<u64> {
    return Some(60 * 60);
}

pub async fn get_categories(
    State(state): State<ClientState>,
    headers: HeaderMap,
) -> ClientResult<Response> {
    let etag_key = get_categories_etag();
    let user_etag = headers.typed_get::<IfNoneMatch>();

    return state.etag_handler.etag_cache_or(
        etag_key, user_etag, get_categories_ttl(),
        || async {
            state.category_repo.get_all().await
        }
    ).await
}

fn get_chart_etag(params: &ChartParams) -> String {
    let category = params.category_id.unwrap_or(0);
    let platform: i32 = params.platform.clone().into();
    let type_id: i32 = params.type_id.clone().unwrap_or(ObjTypeId::Unspecified).into();
    return format!(
        "cache:etag:get_chart:{}:{}:{}:{}:{}",
        platform, type_id,
        category, params.size, params.offset
    );
}

fn get_chart_ttl() -> Option<u64> {
    return Some(60 * 60);
}

pub async fn get_chart(
    State(state): State<ClientState>,
    Query(params): Query<ChartParams>,
    headers: HeaderMap,
) -> ClientResult<Response> {
    if params.size > 100 {
        return Err(ClientError::BadInput(
            "query parameter 'size' max 100".to_string(),
        ));
    }

    let etag_key = get_chart_etag(&params);
    let user_etag = headers.typed_get::<IfNoneMatch>();

    return state.etag_handler.etag_cache_or(
        etag_key, user_etag, get_chart_ttl(),
        || async {
            return match params.category_id {
                Some(cat_id) => {
                    state
                        .object_repo
                        .chart_by_category(
                            params.platform.into(), cat_id,
                            params.size, params.offset
                        )
                        .await
                }
                None => {
                    state
                        .object_repo
                        .chart_by_app_type(
                            params.platform.into(), params.type_id, 
                            params.size, params.offset
                        )
                        .await
                }
            };
        }
    ).await
}
