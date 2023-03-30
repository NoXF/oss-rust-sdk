use oss_rust_sdk::oss::*;
use std::collections::HashMap;
use std::str;
use tokio::runtime::Runtime;

fn main() {
    get_object();
    async_get_object();
    list_object();
    async_list_object();
}

fn get_object() {
    use oss_rust_sdk::object::*;

    // use your own oss config
    let oss_instance = OSS::new(
        "your_AccessKeyId",
        "your_AccessKeySecret",
        "your_Endpoint",
        "your_Bucket",
    );

    let result = oss_instance.get_object("objectName", None::<HashMap<&str, &str>>, None);
    assert_eq!(result.is_ok(), true);
    println!("text = {:?}", String::from_utf8(result.unwrap()));
}

fn async_get_object() {
    use oss_rust_sdk::async_object::*;

    // use your own oss config
    let oss_instance = OSS::new(
        "your_AccessKeyId",
        "your_AccessKeySecret",
        "your_Endpoint",
        "your_Bucket",
    );

    let mut rt = Runtime::new().expect("failed to start runtime");

    rt.block_on(async move {
        let buf = oss_instance
            .get_object("objectName", None::<HashMap<&str, &str>>, None)
            .await
            .unwrap();
        println!("buffer = {:?}", String::from_utf8(buf.to_vec()));
    });
}

fn async_list_object() {
    use oss_rust_sdk::async_object::*;

    // use your own oss config
    let oss_instance = OSS::new(
        "your_AccessKeyId",
        "your_AccessKeySecret",
        "your_Endpoint",
        "your_Bucket",
    );
    let mut params = HashMap::new();
    params.insert("max-keys", Some("5"));

    let mut rt = Runtime::new().expect("failed to start runtime");
    rt.block_on(async move {
        let result = oss_instance.list_object(None, params).await.unwrap();

        for object in result.contents() {
            dbg!(&object.key());
        }
    });
}

fn list_object() {
    use oss_rust_sdk::object::*;

    // use your own oss config
    let oss_instance = OSS::new(
        "your_AccessKeyId",
        "your_AccessKeySecret",
        "your_Endpoint",
        "your_Bucket",
    );

    let mut params = HashMap::new();
    params.insert("max-keys", Some("5"));
    let result = oss_instance.list_object(None, params).unwrap();

    for object in result.contents() {
        dbg!(&object.key());
    }
}
