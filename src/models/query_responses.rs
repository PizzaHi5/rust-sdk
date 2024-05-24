use serde::Deserialize;
use std::fmt::Debug;

use crate::TurnkeyResult;

pub trait QueryResponse: Debug + Send {}

impl QueryResponse for GetWalletResult {}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetWalletResult {
    pub wallet_id: String,
    pub wallet_name: String,
}

// pub fn convert_to_result(option: Option<Box<dyn QueryResponse>>) -> TurnkeyResult<()> {
//     match option {
//         Some(GetWalletResult) => Ok(GetWalletResult),
//         None => Err(TurnkeyError::OtherError),
//     }
// }