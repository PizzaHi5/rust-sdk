use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignRawPayloadResult {
    pub r: String,
    pub s: String,
}