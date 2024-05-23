use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use p256::ecdsa::{Signature, SigningKey};
use p256::ecdsa::signature::Signer;
use p256::FieldBytes;
use serde::Serialize;
use serde::Deserialize;
use thiserror::Error;
use reqwest::Client;
use dotenv::dotenv;
use std::env;


use crate::models::*;
use crate::errors::*;

mod models;
mod errors;

#[derive(Error, Debug, PartialEq)]
pub enum StampError {
    #[error("cannot decode private key: invalid hex")]
    InvalidPrivateKeyString(#[from] hex::FromHexError),
    #[error("cannot load private key: invalid bytes")]
    InvalidPrivateKeyBytes,
}

pub struct TurnkeyApiKey {
    pub private_key_hex: String,
    pub public_key_hex: String,
}

/// Represents the Turnkey service client, encapsulating all necessary keys and the API client.
pub struct TurnkeyClient {
    api_keys: TurnkeyApiKey,
    organization_id: String,
    client: Client,
}

impl TurnkeyClient {

    pub fn new() -> TurnkeyResult<Self> {
        dotenv().ok();

        Ok(Self {
            api_keys: TurnkeyApiKey {
                public_key_hex: env::var("TURNKEY_API_PUBLIC_KEY")?,
                private_key_hex: env::var("TURNKEY_API_PRIVATE_KEY")?,
            },
            organization_id: env::var("TURNKEY_ORGANIZATION_ID")?,
            client: Client::new(),
        })
    }

    pub async fn get_wallet_v1(&self, wallet_id: String) -> TurnkeyResult<String> {
        let sign_get_wallet_payload_body = GetWalletV1 {
            organization_id: self.organization_id.clone(),
            wallet_id: wallet_id,
        };

        let body = serde_json::to_string(&sign_get_wallet_payload_body)?;
        //Note: Make TurnkeyResults implement StampError
        let x_stamp = self.stamp(&body, &self.api_keys).unwrap();

        let response = self
            .client
            .post("https://api.turnkey.com/public/v1/query/get_wallet")
            .header("Content-Type", "application/json")
            .header("X-Stamp", &x_stamp)
            .body(body)
            .send()
            .await;

        let response_body = self.process_response::<ActivityResponse>(response).await?;

        if let Some(result) = response_body.activity.result {
            if let Some(result) = result.get_wallet_results {
                let wallet_result = format!("{}{}", result.wallet_id, result.wallet_name);

                return Ok(wallet_result);
            }
        }

        return Err(TurnkeyError::OtherError(
            "Missing GET_WALLET_PAYLOAD result".into(),
        ))
    }

    fn stamp(&self, request_body: &String, api_key: &TurnkeyApiKey) -> Result<String, StampError> {
        let private_key_bytes = hex::decode(&api_key.private_key_hex)?;
        let signing_key: SigningKey = SigningKey::from_bytes(FieldBytes::from_slice(&private_key_bytes)).map_err(|_| StampError::InvalidPrivateKeyBytes)?;
        let sig: Signature = signing_key.sign(request_body.as_bytes());
    
        let stamp = ApiStamp {
            public_key: api_key.public_key_hex.clone(),
            signature: hex::encode(sig.to_der()),
            scheme: "SIGNATURE_SCHEME_TK_API_P256".into(),
        };
    
        let json_stamp = serde_json::to_string(&stamp).unwrap();
    
        Ok(BASE64_URL_SAFE_NO_PAD.encode(json_stamp.as_bytes()))
    }

    async fn process_response<T>(
        &self,
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> TurnkeyResult<T>
    where
        T: for<'de> Deserialize<'de> + 'static,
    {
        match response {
            Ok(res) => {
                if res.status().is_success() {
                    // On success, deserialize the response into the
                    // expected type T
                    res.json::<T>().await.map_err(TurnkeyError::HttpError)
                } else {
                    // On failure, attempt to deserialize into the error
                    // response type
                    let error_res = res.json::<TurnkeyResponseError>().await;
                    error_res
                        .map_err(TurnkeyError::HttpError)
                        .and_then(|error| Err(TurnkeyError::MethodError(error)))
                }
            }
            Err(e) => {
                // On a reqwest error, convert it into a
                // TurnkeyError::HttpError
                Err(TurnkeyError::HttpError(e))
            }
        }
    }
}