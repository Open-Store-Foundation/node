use alloy::primitives::{Address, Bytes, B256, U256};
use alloy::sol_types::SolEvent;
use codegen_contracts::contracts::AppBuildsPluginV1::AppBuild;
use codegen_contracts::contracts::{App, AppBuildsPluginV1, AppDistributionPluginV1, AppOwnerPluginV1, DevAccount, DevAccountAppsPluginV1};
use net_client::node::provider::Web3Provider;
use net_client::node::result::EthResult;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ObjOwnerData {
    pub website: String,
    pub fingerprints: Vec<B256>,
    pub proofs: Vec<Bytes>,
}

pub struct ScObjService {
    provider: Arc<Web3Provider>
}

impl ScObjService {
    
    pub const APP_CREATED_HASH: B256 = DevAccountAppsPluginV1::AppCreated::SIGNATURE_HASH;
    
    pub fn new(client: Arc<Web3Provider>) -> Self {
        Self { provider: client }
    }

    // TODO optimize
    pub async fn get_owner_name(&self, obj: Address) -> EthResult<String> {
        let contract = App::new(obj, &self.provider);
        let dev_address = contract.developer().call()
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

    pub async fn get_last_owner_data(&self, app: Address) -> EthResult<ObjOwnerData> {
        let contract = AppOwnerPluginV1::new(app, &self.provider);
        let data = contract.getState_0()
            .call()
            .await?;

        return Ok(
            ObjOwnerData {
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

    pub async fn get_owner_data(&self, app: Address, owner_version: u64) -> EthResult<ObjOwnerData> {
        let contract = AppOwnerPluginV1::new(app, &self.provider);

        let data = contract.getState_1(U256::from(owner_version))
            .call()
            .await?;

        return Ok(
            ObjOwnerData {
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
