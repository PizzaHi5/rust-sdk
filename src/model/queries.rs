use serde::{Deserialize, Serialize};

// Queries only covers wallet and private keys currently
//pub type Query//

// Constrain query inputs to fit this trait
pub trait Queryable {}

impl Queryable for GetWallet {}
impl Queryable for ListWalletAccounts {}
impl Queryable for ListWallets {}
impl Queryable for GetPrivateKey {}
impl Queryable for ListPrivateKeys {}

//
// Wallet Queries
//
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetWallet {
    #[serde(rename = "type")]
    pub organization_id: String,
    pub wallet_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListWalletAccounts {
    #[serde(rename = "type")]
    pub organization_id: String,
    pub wallet_id: String,
    pub pagination_options: PaginationOptions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
    limit: String,
    before: String,
    after: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListWallets {
    #[serde(rename = "type")]
    pub organization_id: String,
}

//
// Private Key Queries
//
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPrivateKey {
    #[serde(rename = "type")]
    pub organization_id: String,
    pub private_key_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListPrivateKeys {
    #[serde(rename = "type")]
    pub organization_id: String,
}
