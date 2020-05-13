use std::str;

use oss_rust_sdk::prelude::*;
use reqwest::Error;
use tokio::runtime::Runtime;

fn main() {
    println!("Hello, world!");
    sync_get_object_demo();
    async_get_object_demo();
}

fn sync_get_object_demo() {
    let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
    let result = oss_instance.get_object("objectName", None, None);
    assert_eq!(result.is_ok(), true);
    println!("buffer = {:?}", String::from_utf8(result.unwrap()));
}

fn async_get_object_demo() {
    let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");

    let mut rt = Runtime::new().expect("failed to start runtime");

    rt.block_on(async move {
        let _result = oss_instance.async_get_object("objectName", None, None).await.unwrap();
        println!("buffer = {:?}", String::from_utf8(result.unwrap()));
    });
}
