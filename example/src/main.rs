use std::str;

use futures::Future;
use oss_rust_sdk::prelude::*;
use reqwest::Error;
use tokio_core::reactor::Core;

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
    let mut core = Core::new().unwrap();
    let _handle = core.handle();

    let async_result = core.run(wrap("objectName")).unwrap();
    println!("async_result = {}", async_result);
}

fn wrap(bucket_name: &str) -> impl Future<Item=String, Error=(Error)> {
    let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
    let ret = oss_instance.async_get_object(bucket_name, None, None)
        .map(|body| {
            str::from_utf8(&body).unwrap().to_string()
        });
    ret
}
