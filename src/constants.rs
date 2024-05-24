use lazy_static::lazy_static;
use std::collections::HashMap;

pub const BASE_URL: &str = "https://api.turnkey.com/";
pub const QUERY_URL: &str = "public/v1/query/";
pub const SUBMIT_URL: &str = "public/v1/submit/";
// How to use: Pass in key ref string to request
// Mapping cannot be changed after initialization
lazy_static! {
    static ref FUNCTIONS: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("get_wallet", "public/v1/query/get_wallet");
        map.insert("list_wallet_accounts", "public/v1/query/list_wallet_accounts");
        map
        //example uri = public/{version}/{type}/{function}, Vec -> version, type and function
        // use a const slice instead of Vec
    };
}
