use alloy::primitives::{Address, Bytes, Log, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::sol_types::{SolCall, SolEvent};
use codegen_contracts::contracts::AppBuildsPluginV1::AppBuild;
use codegen_contracts::contracts::{App, AppBuildsPluginV1, AppDistributionPluginV1, AppOwnerPluginV0, AppOwnerPluginV1, DevAccount, DevAccountAppsPluginV1};
use net_client::node::provider::Web3Provider;
use net_client::node::result::EthResult;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ObjOwnerProofV0 {
    pub data: ObjOwnerDataV1,
    pub certs: Vec<Bytes>,
    pub proofs: Vec<Bytes>,
}

#[derive(Debug, Clone)]
pub struct ObjOwnerDataV1 {
    pub website: String,
    pub fingerprints: Vec<B256>,
    pub proofs: Vec<Bytes>,
}

pub struct ScObjService {
    provider: Arc<Web3Provider>
}

impl ScObjService {
    
    pub const APP_CREATED_HASH: B256 = DevAccountAppsPluginV1::AppCreated::SIGNATURE_HASH;
    pub const APP_OWNER_CHANGED_HASH: B256 = AppOwnerPluginV0::AppOwnerChanged::SIGNATURE_HASH;

    pub fn new(client: Arc<Web3Provider>) -> Self {
        Self { provider: client }
    }

    pub fn decode_owner_changed_log(data: &Log) -> alloy::sol_types::Result<Log<AppOwnerPluginV0::AppOwnerChanged>> {
        let result = AppOwnerPluginV0::AppOwnerChanged::decode_log(data);
        return result;
    }

    // TODO optimize
    pub async fn get_owner_name(&self, obj: Address) -> EthResult<String> {
        let contract = App::new(obj, &self.provider);
        let dev_address = contract.owner().call()
            .await?;
        
        let dev_account = DevAccount::new(dev_address, &self.provider);
        let name = dev_account.getName().call()
            .await?;
        
        return Ok(name);
    }
    
    pub async fn get_general_info(&self, obj: Address) -> EthResult<App::AppGeneralInfo> {
        let contract = App::new(obj, &self.provider);
        let build = contract.getGeneralInfo().call()
            .await?;

        return Ok(build);
    }
    
    pub async fn get_artifact(&self, obj: Address, build_version: i64) -> EthResult<AppBuild> {
        let contract = AppBuildsPluginV1::new(obj, &self.provider);
        let build = contract.getBuild(U256::from(build_version)).call()
            .await?;

        return Ok(build);
    }

    pub async fn website(&self, obj: Address, version: u64) -> EthResult<String> {
        let contract = AppOwnerPluginV1::new(obj, &self.provider);

        let website = contract.domain_0(U256::from(version))
            .call()
            .await?;

        return Ok(website);
    }

    pub async fn get_last_owner_data(&self, app: Address) -> EthResult<ObjOwnerDataV1> {
        let contract = AppOwnerPluginV1::new(app, &self.provider);
        let data = contract.getState_0()
            .call()
            .await?;

        return Ok(
            ObjOwnerDataV1 {
                website: data.domain,
                fingerprints: data.fingerprints,
                proofs: data.proofs,
            }
        );
    }

    pub async fn get_distribution(&self, obj: Address) -> EthResult<Vec<String>> {
        let contract = AppDistributionPluginV1::new(obj, &self.provider);
        
        let data = contract.getDistribution()
            .call()
            .await?
            .sources
            .into_iter()
            .map(|source| String::from_utf8_lossy(source.as_ref()).to_string())
            .collect::<Vec<String>>();

        return Ok(data);
    }

    pub async fn get_owner_data_v0(&self, app: Address, owner_version: u64) -> EthResult<ObjOwnerDataV1> {
        let contract = AppOwnerPluginV0::new(app, &self.provider);

        let data = contract.getState_1(U256::from(owner_version))
            .call()
            .await?;

        return Ok(
            ObjOwnerDataV1 {
                website: data.domain,
                fingerprints: data.fingerprints,
                proofs: vec![],
            }
        );
    }

    pub async fn get_owner_proof_v0(&self, app: Address, owner_version: u64) -> EthResult<Option<ObjOwnerProofV0>> {
        let contract = AppOwnerPluginV0::new(app, &self.provider);

        let data = contract.getState_1(U256::from(owner_version))
            .call()
            .await?;

        let block_number = data.blockNumber.to::<u64>();
        if block_number == 0 {
            return Ok(None);
        }

        let filter = Filter::new()
            .address(app)
            .from_block(block_number)
            .to_block(block_number)
            .event_signature(Self::APP_OWNER_CHANGED_HASH);

        let logs = self.provider.get_logs(&filter)
            .await?;

        let log = logs.into_iter()
            .find_map(|l| {
                let owner_log = Self::decode_owner_changed_log(l.as_ref());
                if let Ok(data) = owner_log {
                    if data.data.version != owner_version { // TODO combine with let when ready
                        return None;
                    }

                    return Some(data.data);
                };

                return None;
            });

        return if let Some(owner_data) = log {
            Ok(Some(
                ObjOwnerProofV0 {
                    data: ObjOwnerDataV1 {
                        website: data.domain,
                        fingerprints: data.fingerprints,
                        proofs: vec![],
                    },
                    proofs: owner_data.proofs,
                    certs: owner_data.pubKeys,
                }
            ))
        } else {
            Ok(None)
        };
    }

    pub async fn get_owner_data_v1(&self, app: Address, owner_version: u64) -> EthResult<ObjOwnerDataV1> {
        let contract = AppOwnerPluginV1::new(app, &self.provider);

        let data = contract.getState_1(U256::from(owner_version))
            .call()
            .await?;

        return Ok(
            ObjOwnerDataV1 {
                website: data.domain,
                fingerprints: data.fingerprints,
                proofs: data.proofs,
            }
        );
    }

    pub async fn obj_package(&self, obj: Address) -> EthResult<String> {
        let contract = App::new(obj, &self.provider);
        let package = contract.getId()
            .call()
            .await?;

        return Ok(package);
    }
}

#[derive(Debug, Clone)]
pub struct ObjInfo {
    pub logo: String,
    pub name: String,
    pub package_name: String,
    pub description: String,

    pub platform_id: i32,
    pub category_id: i32,
}

impl ObjInfo {
    pub fn new(logo: String, name: String, package_name: String, description: String, platform_id: i32, category_id: i32) -> Self {
        Self { logo, name, package_name, description, platform_id, category_id }
    }
}
