use std::str;
use oss_rust_sdk::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

fn main() {
    get_object();
    async_get_object();
}

fn get_object() {
    // use your own oss config
    let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");

    let result = oss_instance.get_object("objectName", None::<HashMap<&str, &str>>, None);
    assert_eq!(result.is_ok(), true);
    println!("text = {:?}", String::from_utf8(result.unwrap()));
}

fn async_get_object() {
    // use your own oss config
    let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");

    let mut rt = Runtime::new().expect("failed to start runtime");

    rt.block_on(async move {
        let buf = oss_instance.async_get_object("objectName", None, None).await.unwrap();
        println!("buffer = {:?}", String::from_utf8(buf.to_vec()));
    });
}

