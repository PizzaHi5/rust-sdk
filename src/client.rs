use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use dotenv::dotenv;
use models::queries::Queryable;
use models::query_responses::QueryResponse;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use p256::FieldBytes;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use thiserror::Error;
use url::Url;

use crate::errors::*;
use crate::models::*;
use crate::constants::*;
use crate::activity::*;

mod constants;
mod errors;
pub mod models;
//move or remove this
#[derive(Error, Debug, PartialEq)]
pub enum StampError {
    #[error("cannot decode private key: invalid hex")]
    InvalidPrivateKeyString(#[from] hex::FromHexError),
    #[error("cannot load private key: invalid bytes")]
    InvalidPrivateKeyBytes,
}

/// Holds api key information
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
    /// Creates a new instance of the Turnkey client.
    ///
    /// # Examples
    ///
    /// ```
    /// use turnkey::client::Turnkey;
    ///
    /// let turnkey_client = Turnkey::new();
    /// ```
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
    // pub async fn get_wallet_v1(&self, wallet_id: String) -> TurnkeyResult<GetWalletResult> {
    //     //construct input data type
    //     let sign_get_wallet_payload_body = GetWalletV1 {
    //         organization_id: self.organization_id.clone(),
    //         wallet_id,
    //     };

    //     //prepare x-stamp
    //     let body = serde_json::to_string(&sign_get_wallet_payload_body)?;
    //     //   Note: Make TurnkeyResults implement StampError
    //     let x_stamp = self.stamp(&body).unwrap();

    //     //send stamped body
    //     let response = self
    //         .client
    //         .post("https://api.turnkey.com/public/v1/query/get_wallet")
    //         .header("Content-Type", "application/json")
    //         .header("X-Stamp", &x_stamp)
    //         .body(body)
    //         .send()
    //         .await;
    //     //handle response
    //     let response_body = self.process_response::<ActivityResponse>(response).await?;
    //     //format response data and return 
    //     if let Some(result) = response_body.activity.result {
    //         if let Some(result) = result.get_wallet_results {
    //             let get_wallet_result = GetWalletResult {
    //                 wallet_id: result.wallet_id,
    //                 wallet_name: result.wallet_name,
    //             };
    //             return Ok(get_wallet_result);
    //         }
    //     }

    //     Err(TurnkeyError::OtherError(
    //         "Missing GET_WALLET_PAYLOAD result".into(),
    //     ))
    // }

    /// Sends a GET request to the Turnkey API
    ///
    /// Reference the model/queries for valid value types
    /// 
    /// # Examples
    ///
    /// ```
    /// use turnkey::client::Turnkey;
    /// use turnkey::model::{queries, query_responses};
    /// let turnkey_client = Turnkey::new();
    /// let input = GetWallet {
    ///     organization_id: "org_id",
    ///     wallet_id: "wallet_id",
    /// };
    /// let get_response = turnkey_client.query_request(input, "get_wallet");
    /// ```
    pub async fn query_request<T, U>(&self, value: &T, method: &str) -> TurnkeyResult<(U, TurnkeyError)> 
    where 
        T: Serialize + Queryable,
        U: Serialize + QueryResponse,
    {
        //check valid str input

        //construct url, sign, and stamp 
        let url = Url::parse(&format!("{}{}{}", BASE_URL, QUERY_URL, method))?;
        let sign_and_stamp = self.sign_and_stamp(value)?;

        //send request
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Stamp", &sign_and_stamp.x_stamp)
            .body(sign_and_stamp.request_body)
            .send()
            .await;
        
        let response_body = self.process_response::<ActivityResponse>(response).await?;

        if let Some(result) = response_body.activity.result {
            //need to handle multiple result types
            // if let Some(result) = result.QueryResponse {
            //      match result
            //      
            //      return Ok(signature_bytes);
            // }
        }

        Err(TurnkeyError::OtherError(
            "Missing GET_WALLET_PAYLOAD result".into(),
        ))
    }

    // POST
    pub async fn submit_request<T: Serialize>(&self, value: &T, method: &str) -> TurnkeyResult<()> {
        Ok(())
    }

    // Send serializable type and stamp it, body is Queryable or Submittable
    fn sign_and_stamp<T: Serialize>(&self, body: &T) -> TurnkeyResult<BodyAndStamp> {
        let request_body = serde_json::to_string(&body)?;
        // Note: Make TurnkeyResults implement StampError
        let x_stamp = self.stamp(&request_body).unwrap();
        Ok(BodyAndStamp {
            request_body,
            x_stamp,
        })
    }
    /// Creates a digital stamp for a given message.
    ///
    /// This method signs a given message with a private API key, generates a
    /// signature, and constructs a digital stamp containing the signature,
    /// the public API key, and the signature scheme. The digital stamp is
    /// then serialized, base64-url encoded, and returned.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be signed and stamped.
    ///
    fn stamp(&self, message: &String) -> Result<String, StampError> {
        let private_key_bytes = hex::decode(&self.api_keys.private_key_hex)?;
        let signing_key: SigningKey = 
            SigningKey::from_bytes(FieldBytes::from_slice(&private_key_bytes))
                .map_err(|_| StampError::InvalidPrivateKeyBytes)?;
        let sig: Signature = signing_key.sign(message.as_bytes());
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
