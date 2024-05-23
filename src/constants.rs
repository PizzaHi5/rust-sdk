use lazy_static::lazy_static;
use std::collections::HashMap;

pub const BASE_URL: &str = "https://api.turnkey.com/";

lazy_static! {
    static ref FUNCTIONS: HashMap<&'static str, Vec<&'static str>> = {
        let mut map = HashMap::new();
        map.insert("hi", vec!["no"]);
        map
        //example uri = public/{version}/{type}/{function}, Vec -> version, type and function
        // use a const slice instead of Vec
    };
}
