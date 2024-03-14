use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use constants::BASE_URL;
use dotenv::dotenv;
use model::queries::Queryable;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use p256::FieldBytes;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use thiserror::Error;
use url::Url;

use crate::errors::*;
use crate::models::*;
use crate::model::*;

mod constants;
mod errors;
mod models;
mod model;

//move or remove this
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
    example_key_info: KeyInfo,
    client: Client,
}

/// Holds the private key ID and corresponding public key for a specific operation.
#[derive(Clone)]
pub struct KeyInfo {
    private_key_id: String,
    public_key: Pubkey,
}

/// Enumerates the selectable keys for operations, distinguishing by their use case.
pub enum KeySelector {
    ExampleKey,
    // other key info variants depending on what other keys you need to sign with
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

    //abstract this to a request function
    pub async fn get_wallet_v1(&self, wallet_id: String) -> TurnkeyResult<GetWalletResult> {
        //construct input data type
        let sign_get_wallet_payload_body = GetWalletV1 {
            organization_id: self.organization_id.clone(),
            wallet_id,
        };

        //prepare x-stamp
        let body = serde_json::to_string(&sign_get_wallet_payload_body)?;
        //   Note: Make TurnkeyResults implement StampError
        let x_stamp = self.stamp(&body).unwrap();

        //send stamped body
        let response = self
            .client
            .post("https://api.turnkey.com/public/v1/query/get_wallet")
            .header("Content-Type", "application/json")
            .header("X-Stamp", &x_stamp)
            .body(body)
            .send()
            .await;

        let response_body = self.process_response::<ActivityResponse>(response).await?;

        //handle response
        let response_body = self.process_response::<ActivityResponse>(response).await?;
        //format response data and return 
        if let Some(result) = response_body.activity.result {
            if let Some(result) = result.get_wallet_results {
                let get_wallet_result = GetWalletResult {
                    wallet_id: result.wallet_id,
                    wallet_name: result.wallet_name,
                };
                return Ok(get_wallet_result);
            }
        }

        Err(TurnkeyError::OtherError(
            "Missing GET_WALLET_PAYLOAD result".into(),
        ))
    }

    //define a new trait for all requests : Wrap it to one type, subtype of serialize
    pub async fn query_request<T: Serialize + Queryable>(&self, value: &T, function: &str) -> TurnkeyResult<()> {
        //check valid str input

        //
        
        Ok(())
    }

    pub async fn submit_request<T: Serialize>(&self, value: &T) -> TurnkeyResult<()> {
        Ok(())
    }

    pub fn sign_and_stamp<T: Serialize>(&self, body: &T) -> TurnkeyResult<BodyAndStamp> {
        let request_body = serde_json::to_string(&body)?;
        // Note: Make TurnkeyResults implement StampError
        let x_stamp = self.stamp(&request_body).unwrap();
        Ok(BodyAndStamp {
            request_body,
            x_stamp,
        })
    }

    //example uri = public/{version}/{type}/{function}
    // version = v1
    // type = query or submit
    // function = specific to api, store a premade list
    fn construct_url(
        &self, 
        uri: &str, 
        query: &HashMap<&str, Vec<&str>>, 
        substitution: &HashMap<&str, &str>,
    ) -> Url {
        //Substitute placeholders in the URI
        let substituted_uri = self.substitute_path(uri, substitution);
        //Create a new URL object
        let mut url = Url::parse(BASE_URL).expect("Invalid base URL");
        url.set_path(&substituted_uri);
        //Append query parameters
        for (key, values) in query {
            for value in values {
                url.query_pairs_mut().append_pair(key, value);
            }
        }
        url
    }

    // replaces url path segments
    fn substitute_path(&self, uri: &str, substitution_map: &HashMap<&str, &str>) -> String {
        let mut result = uri.to_string();
        for (key, value) in substitution_map {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        if result.contains('{') && result.contains('}') {
            panic!(
                "Substitution error: found unsubstituted components in \"{}\"", 
                result
            );
        }
        result
    }
    //modify Result error to align with TurnkeyError
    fn stamp(&self, request_body: &String) -> Result<String, StampError> {
        let private_key_bytes = hex::decode(&self.api_keys.private_key_hex)?;
        let signing_key: SigningKey = 
            SigningKey::from_bytes(FieldBytes::from_slice(&private_key_bytes))
                .map_err(|_| StampError::InvalidPrivateKeyBytes)?;
        let sig: Signature = signing_key.sign(request_body.as_bytes());
        let stamp = ApiStamp {
            public_key: self.api_keys.public_key_hex.clone(),
            signature: hex::encode(sig.to_der()),
            scheme: "SIGNATURE_SCHEME_TK_API_P256",
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
