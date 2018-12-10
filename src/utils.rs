use std::fs::File;
use std::io::{BufReader, Read};
use failure::Error;
use reqwest::header::{HeaderMap, HeaderName};
use std::collections::HashMap;

#[inline]
pub fn load_file_to_string(p: &str) -> Result<String, Error> {
    let f = File::open(p)?;
    let mut f = BufReader::new(f);
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

pub fn to_headers(hashmap: HashMap<&str, &str>) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();
    for (key, val) in hashmap.iter() {
        headers.insert(HeaderName::from_bytes(key.as_bytes())?, val.parse()?);
    }
    Ok(headers)
}
