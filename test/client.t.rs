use turnkey::{errors::TurnkeyResult, KeySelector, Turnkey};
use turnkey::client::Turnkey;
use dotenv::dotenv;
//use mockito::Server;

// Testing
#[cfg(test)]
mod test {
    use super::*;
    use serde_json::Value;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_example_wallet() -> TurnkeyResult<()> {
        dotenv().ok();
        let mut server = mockito::Server::new();

        let turnkey_client = Turnkey::new();

        Ok(())
    }

    #[test]
    fn test_stamps() {
        let stamp = stamp(
            "hello from TKHQ".to_string(), 
            &TurnkeyApiKey { private_key_hex: "9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69".to_string(), public_key_hex: "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45".to_string()},
        ).unwrap();

        // The stamp should be valid base64
        let decoded_stamp_bytes = BASE64_URL_SAFE_NO_PAD.decode(stamp).unwrap();
        
        // These bytes should be valid UTF8 characters
        let decoded_stamp_string = String::from_utf8(decoded_stamp_bytes).unwrap();
        
        // The resulting string should be valid JSON
        let json_stamp: Value = serde_json::from_str(&decoded_stamp_string).unwrap();

        // And finally: the signature scheme and public key should be correct
        assert_eq!(json_stamp["scheme"], "SIGNATURE_SCHEME_TK_API_P256");
        assert_eq!(json_stamp["publicKey"], "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45");
    }

    #[test]
    fn test_bad_hex() {
        let err = stamp(
            "body".to_string(),
            &TurnkeyApiKey { private_key_hex: "bad-private-key".to_string(), public_key_hex: "".to_string()},
        ).unwrap_err();
        assert_eq!(format!("{:?}", err), "InvalidPrivateKeyString(OddLength)".to_string());
        assert_eq!(err.to_string(), "cannot decode private key: invalid hex".to_string());
    }

    #[test]
    fn test_bad_bytes() {
        let err = stamp(
            "body".to_string(),
            &TurnkeyApiKey { private_key_hex: "fffffffff61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c70".to_string(), public_key_hex: "".to_string()},
        ).unwrap_err();
        assert_eq!(format!("{:?}", err), "InvalidPrivateKeyBytes".to_string());
        assert_eq!(err.to_string(), "cannot load private key: invalid bytes".to_string());
    }

}