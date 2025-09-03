use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    DevAccount,
    "./contracts/PublisherAccount.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    DevAccountAppsPluginV1,
    "./contracts/PublisherAccountAppsPluginV1.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    App,
    "./contracts/AppAsset.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AppOwnerPluginV1,
    "./contracts/AppOwnerPluginV5.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AppDistributionPluginV1,
    "./contracts/AppDistributionPluginV1.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AppBuildsPluginV1,
    "./contracts/AppBuildsPluginV5.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AssetlinksOracle,
    "./contracts/AssetlinksOracleV5.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    OpenStore,
    "./contracts/OpenStoreV5.json"
);