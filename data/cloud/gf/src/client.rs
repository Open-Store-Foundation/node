use crate::data::{BucketMetaHead, HeadObjectMeta, SpProviders, VirtualGroupsFamily};
use crate::json_rpc::{JsonRpcParams, JsonRpcRequest, JsonRpcResponse, JsonRpcResponseData};
use crate::proto::{QueryHeadObjectRequest, QueryHeadObjectResponse};
use crate::r#const::{MS_AUTH_OFFSET, SHA256_EMPTY};
use alloy::primitives::keccak256;
use chrono::TimeDelta;
use core_std::time::{current_time, Iso8601};
use lazy_static::lazy_static;
use net_client::http::HttpClient;
use net_client::node::signer::ValidatorSigner;
use prost::{DecodeError, Message};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Method, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, warn};
use url::{ParseError, Url};

const HTTP_HEADER_AUTHORIZATION: &str = "Authorization";

const HTTP_HEADER_EXPIRY_TIMESTAMP: & str = "X-Gnfd-Expiry-Timestamp";
const HTTP_HEADER_CONTENT_SHA256: & str = "X-Gnfd-Content-Sha256";
const HTTP_HEADER_TRANSACTION_HASH: & str = "X-Gnfd-Txn-Hash";
const HTTP_HEADER_OBJECT_ID: & str = "X-Gnfd-Object-ID";
const HTTP_HEADER_REDUNDANCY_INDEX: & str = "X-Gnfd-Redundancy-Index";
const HTTP_HEADER_RESOURCE: & str = "X-Gnfd-Resource";
const HTTP_HEADER_DATE: & str = "X-Gnfd-Date";
const HTTP_HEADER_RANGE: & str = "Range";
const HTTP_HEADER_PIECE_INDEX: & str = "X-Gnfd-Piece-Index";
const HTTP_HEADER_CONTENT_TYPE: & str = "Content-Type";
const HTTP_HEADER_CONTENT_MD5: & str = "Content-MD5";
const HTTP_HEADER_UNSIGNED_MSG: & str = "X-Gnfd-Unsigned-Msg";
const HTTP_HEADER_USER_ADDRESS: & str = "X-Gnfd-User-Address";

lazy_static! {
    pub static ref SUPPORTED_HEADERS: Vec<String> = vec![
        HTTP_HEADER_CONTENT_SHA256.to_lowercase(),
        HTTP_HEADER_TRANSACTION_HASH.to_lowercase(),
        HTTP_HEADER_OBJECT_ID.to_lowercase(),
        HTTP_HEADER_REDUNDANCY_INDEX.to_lowercase(),
        HTTP_HEADER_RESOURCE.to_lowercase(),
        HTTP_HEADER_DATE.to_lowercase(),
        HTTP_HEADER_EXPIRY_TIMESTAMP.to_lowercase(),
        HTTP_HEADER_RANGE.to_lowercase(),
        HTTP_HEADER_PIECE_INDEX.to_lowercase(),
        HTTP_HEADER_CONTENT_TYPE.to_lowercase(),
        HTTP_HEADER_CONTENT_MD5.to_lowercase(),
        HTTP_HEADER_UNSIGNED_MSG.to_lowercase(),
        HTTP_HEADER_USER_ADDRESS.to_lowercase(),
    ];
}

type Headers<'a> = HashMap<&'a str, String>;

type GfResult<T> = Result<T, GfError>;

#[derive(Debug, Error)]
pub enum GfError {
    #[error("GfError.ReqwestError: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("GfError.Url: {0}")]
    Url(#[from] ParseError),
    #[error("GfError.Serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("GfError.Serde: {0}")]
    Signers(#[from] alloy::signers::Error),
    #[error("GfError.RpcError: {0}, {1}")]
    RpcError(u16, String),
    #[error("GfError.SpNotFound")]
    SpNotFound,
    #[error("GfError.ResponseFormat")]
    ResponseFormat,
    #[error("Failed to decode protobuf message")]
    DecodeErrorProto(#[from] DecodeError),
    #[error("Failed to decode protobuf message")]
    DecodeErrorBase(#[from] base64::DecodeError),
}

pub struct GreenfieldClient {
    client: HttpClient,
    node_url: String,
    pk: Option<Arc<ValidatorSigner>>,
}

impl GreenfieldClient {
    pub fn new(client: HttpClient, node_url: String, pk: Option<Arc<ValidatorSigner>>) -> Self {
        Self { client, node_url, pk }
    }

    pub fn provider_id(&self) -> u8 {
        return 0;
    }
}

impl GreenfieldClient {

    pub async fn get_object(&self, object_id: &str) -> GfResult<Response> {
        let meta = self.get_object_meta_by_id(object_id)
            .await?;

        let object_name = &meta.object_info.object_name;
        let bucket_name = &meta.object_info.bucket_name;

        let endpoint = self.get_sp_endpoint_by_bucket(bucket_name)
            .await?;

        let parsed = Url::parse(&endpoint)?;
        let host = parsed.authority();
        let url = format!("https://{bucket_name}.{host}/{object_name}");
        let url = Url::parse(url.as_str())?;

        let header_map = self.request_headers(Method::GET, &url)
            .await?;

        let builder = self.client.request(Method::GET, url);
        let response_obj = self.execute(builder, Some(header_map))
            .await?;

        return Ok(response_obj);
    }
    
    pub async fn get_object_logo_info(&self, bucket_name: &String, package_name: &String) -> GfResult<Option<String>> {
        let formatted_package = package_name.replace('.', "_");
        let object_name = format!("open-store-external/{}/logo.png", formatted_package);
        
        let result = self.get_object_meta_by_name(
            bucket_name.to_lowercase(), object_name
        ).await?;
        
        let Some(response) = result else {
            return Ok(None);
        };
        
        let Some(info) = response.object_info else {
            return Ok(None);
        };

        let endpoint = self.get_sp_endpoint_by_bucket(info.bucket_name.as_str())
            .await?;
        
        let logo_url = format!("{}/view/{}/{}", endpoint, info.bucket_name, info.object_name);
        
        return Ok(Some(logo_url));
    }

    pub async fn get_object_meta_by_name(&self, bucket_name: String, object_name: String) -> GfResult<Option<QueryHeadObjectResponse>> {
        let request = QueryHeadObjectRequest {
            bucket_name,
            object_name,
        };

        let data = hex::encode(request.encode_to_vec());

        let rpc = JsonRpcRequest {
            id: 0,
            jsonrpc: "2.0".to_string(),
            method: "abci_query".to_string(),
            params: JsonRpcParams {
                path: "/greenfield.storage.Query/HeadObject".to_string(),
                data,
                prove: false,
            },
        };
        
        let response: JsonRpcResponse<JsonRpcResponseData> = self.post_json(&rpc)
            .await?;
        
        let Some(value) = response.result.response.value else {
            return Ok(None);
        };
        
        let result = base64::decode(value.as_bytes())?;
        let obj = QueryHeadObjectResponse::decode(result.as_slice())?;
        
        return Ok(Some(obj));
    }

    pub async fn get_object_meta_by_id(&self, object_id: &str) -> GfResult<HeadObjectMeta> {
        return self.get_json(format!("storage/head_object_by_id/{object_id}"))
            .await;
    }

    async fn get_sp_endpoint_by_bucket(&self, bucket_name: &str) -> GfResult<String> {
        let info = self.head_bucket(bucket_name)
            .await?;

        let providers = self.get_storage_providers()
            .await?;

        let group = self.get_virtual_group(info.bucket_info.global_virtual_group_family_id)
            .await?;

        let sp_id = group.global_virtual_group_family.primary_sp_id;
        let item = providers.sps.iter()
            .find(|item| item.id == sp_id);

        let Some(sp) = item else {
            return Err(GfError::SpNotFound);
        };

        return Ok(sp.endpoint.clone());
    }

    async fn get_virtual_group(&self, virtual_group: i32) -> GfResult<VirtualGroupsFamily> {
        return self.get_json(format!("virtualgroup/global_virtual_group_family?family_id={virtual_group}"))
            .await;
    }

    async fn get_storage_providers(&self) -> GfResult<SpProviders> {
        return self.get_json("storage_providers".to_string())
            .await;
    }

    async fn head_bucket(&self, bucket_name: &str) -> GfResult<BucketMetaHead> {
        return self.get_json(format!("storage/head_bucket/{bucket_name}"))
            .await;
    }

    ///////////////////
    // Headers
    ///////////////////

    async fn request_headers(
        &self, method: Method, url: &Url
    ) -> GfResult<HeaderMap> {
        let mut headers = self.default_request_headers(None);

        if let Some(ref pk) = self.pk {
            let auth_token = self.auth_token(method, &url, &headers, pk)
                .await?;

            headers.insert(HTTP_HEADER_AUTHORIZATION, auth_token);
        }

        let header_map = self.to_header_map(&headers);

        return Ok(header_map);
    }

    fn default_request_headers(&self, content: Option<&[u8]>) -> Headers {
        let mut headers: HashMap<&str, String> = HashMap::new();

        let time = current_time();
        let expired = time.add(TimeDelta::seconds(MS_AUTH_OFFSET));
        let now_format = time.to_iso8601();
        let to_format = expired.to_iso8601();

        headers.insert(HTTP_HEADER_DATE, now_format.into());
        headers.insert(HTTP_HEADER_EXPIRY_TIMESTAMP, to_format.into());

        match content {
            Some(_) => {},
            None => {
                headers.insert(HTTP_HEADER_CONTENT_TYPE, "application/octet-stream".to_string());
                headers.insert(HTTP_HEADER_CONTENT_SHA256, SHA256_EMPTY.to_string());
            }
        }

        return headers;
    }

    fn to_header_map(&self, headers: &Headers) -> HeaderMap {
        let mut header_map = HeaderMap::new();

        for (k, v) in headers {
            if let (Ok(value), Ok(key)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
                header_map.insert(value, key);
            } else {
                warn!("Error during parsing header: {}: {}", k, v);
                continue
            };
        }

        return header_map
    }

    ///////////////////
    // Auth
    ///////////////////

    async fn auth_token(&self, method: Method, url: &Url, headers: &Headers<'_>, auth: &Arc<ValidatorSigner>) -> GfResult<String> {
        let auth_content = self.auth_content(method, url, headers);
        let auth_hash = keccak256(auth_content.as_bytes());

        let signature = auth.sign_hash(&auth_hash)
            .await?;

        let mut bytes = Vec::with_capacity(65);
        bytes.extend_from_slice(signature.r().as_le_slice());
        bytes.extend_from_slice(signature.s().as_le_slice());
        bytes.push(signature.v() as u8);

        let sign = hex::encode(bytes.as_slice());
        let auth = format!("GNFD1-ECDSA,Signature={}", sign);

        return Ok(auth);
    }

    fn auth_content(&self, method: Method, url: &Url, headers: &Headers) -> String {
        let ah = self.sort_headers(&headers);
        let ch = self.canonical_headers(&ah, &headers, url.authority());
        let jh = self.join_headers(&ah);

        let http_method = method.as_str();
        let path = url.path();
        let query = url.query().unwrap_or("");

        let mut auth_content = String::new();
        auth_content.push_str(format!("{http_method}\n").as_str());
        auth_content.push_str(format!("{path}\n").as_str());
        auth_content.push_str(format!("{query}\n").as_str());
        auth_content.push_str(format!("{ch}\n").as_str());
        auth_content.push_str(format!("{jh}").as_str());

        return auth_content;
    }

    fn sort_headers(&self, headers: &Headers) -> Vec<String> {
        let mut auth_headers: Vec<String> = vec![];
        for (key, _) in headers {
            if SUPPORTED_HEADERS.contains(&key.to_lowercase()) {
                auth_headers.push(key.to_string())
            }
        }
        auth_headers.sort();

        return auth_headers;
    }

    fn canonical_headers(&self, auth_headers: &Vec<String>, headers: &Headers, host: &str) -> String {
        let mut canonical_headers = String::new();

        for header in auth_headers {
            let concat = format!(
                "{}:{}",
                header.to_lowercase().as_str(),
                headers[header.as_str()],
            );

            canonical_headers.push_str(concat.as_str());
            canonical_headers.push_str("\n");
        }

        canonical_headers.push_str(host);
        canonical_headers.push_str("\n");

        return canonical_headers;
    }

    fn join_headers(&self, ah: &Vec<String>) -> String {
        return ah.iter()
            .map(|a| a.to_lowercase())
            .collect::<Vec<String>>()
            .join(";");
    }

    ///////////////////
    // Request sending
    ///////////////////
    async fn post_json<R : Serialize, T : DeserializeOwned>(&self, body: &R) -> GfResult<T> {
        let url = &self.node_url;
        let url = Url::parse(url.as_str())?;
        debug!("Request to: {}", url.as_str());

        let builder = self.client.request(Method::POST, url)
            .json(body);
        
        let result = self.execute(builder, None)
            .await?
            .json::<T>()
            .await?;

        return Ok(result);
    }

    async fn get_json<T : DeserializeOwned>(&self, path: String) -> GfResult<T> {
        let url = &self.node_url;
        let endpoint = format!(
            "{url}/greenfield/{path}"
        );

        let url = Url::parse(endpoint.as_str())?;
        debug!("Request to: {}", url.as_str());

        let builder = self.client.request(Method::GET, url);
        let result = self.execute(builder, None)
            .await?
            .json::<T>()
            .await?;

        return Ok(result);
    }

    async fn execute(&self, req_builder: RequestBuilder, headers: Option<HeaderMap>) -> GfResult<Response> {
        let mut builder = req_builder;
        
        if let Some(head) = headers {
            builder = builder.headers(head);
        }

        let request = builder.build()?;

        let response = self.client.execute(request)
            .await?;

        let code = response.status().as_u16();

        debug!("Status code for request: {}", code);

        if !(200..299).contains(&code) {
            let text = response.text()
                .await
                .unwrap_or("Empty".into());

            debug!("Error during request, response: {}", text);

            return Err(GfError::RpcError(code, text))
        }

        return Ok(response);
    }
}
