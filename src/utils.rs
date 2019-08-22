use std::fs::File;
use std::io::{BufReader, Read};
use reqwest::header::{HeaderMap, HeaderName};
use std::collections::HashMap;
use super::errors::Error;

#[inline]
pub fn load_file_to_string<S>(p: S) -> Result<String, Error> 
where S: AsRef<str> {
    let p = p.as_ref();
    let f = File::open(p)?;
    let mut f = BufReader::new(f);
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

pub fn to_headers<S>(hashmap: HashMap<S, S>) -> Result<HeaderMap, Error> 
    where S: AsRef<str> {
    let mut headers = HeaderMap::new();
    for (key, val) in hashmap.iter() {
        let key = key.as_ref();
        let val = val.as_ref();
        headers.insert(HeaderName::from_bytes(key.as_bytes())?, val.parse()?);
    }
    Ok(headers)
}
