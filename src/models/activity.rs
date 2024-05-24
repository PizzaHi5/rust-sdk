use serde::{Deserialize, Serialize};
use crate::{
    query_responses::*,
    submit_responses::*,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BodyAndStamp {
    #[serde(rename = "type")]
    pub request_body: String,
    pub x_stamp: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResponse {
    pub activity: Activity,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub organization_id: String,
    pub status: String,
    pub result: Option<ActivityResult>,
    #[serde(rename = "type")]
    pub activity_type: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResult {
    #[serde(rename = "type")]
    #[serde(skip_serializing)]
    //pub response_type: Option<Box<dyn QueryResponse + 'static>>,
    pub did_not_finish: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiStamp {
    pub public_key: String,
    pub signature: String,
    pub scheme: &'static str,
}
