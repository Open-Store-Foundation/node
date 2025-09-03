use alloy::hex;
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, Signature, B256};
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::signers::local::{LocalSigner, PrivateKeySigner};
use alloy::signers::Signer;

pub struct ValidatorSigner {
    signer: LocalSigner<SigningKey>,
}

impl ValidatorSigner {

    pub fn new(pk: String) -> Option<ValidatorSigner> {
        let hex = hex::decode(pk).ok()?;
        let eth_pk = PrivateKeySigner::from_slice(hex.as_slice()).ok()?;
        Some(Self { signer: eth_pk })
    }

    pub fn wallet(&self) -> EthereumWallet {
        return EthereumWallet::from(self.signer.clone())
    }

    pub async fn sign_hash(&self, hash: &B256) -> alloy::signers::Result<Signature> {
        self.signer.sign_hash(&hash).await
    }

    pub fn address(&self) -> Address {
        return self.signer.address();
    }
}


#[tokio::test]
async fn test_address() {
    // let pk = ValidatorSigner::new();
    // let address = pk.address();
    // assert_eq!(address, Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
}