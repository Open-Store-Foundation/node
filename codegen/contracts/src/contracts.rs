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
    "./contracts/AppOwnerPluginV1.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AppOwnerPluginV0,
    "./contracts/AppOwnerPluginV0_2.json"
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
    "./contracts/AppBuildsPluginV1.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AssetlinksOracle,
    "./contracts/AssetlinksOracle.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    OpenStore,
    "./contracts/OpenStore.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    OpenStoreV0,
    "./contracts/OpenStoreV0.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    TrustedMulticall,
    "./contracts/TrustedMulticall.json"
);