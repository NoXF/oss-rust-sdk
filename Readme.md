It's an unofficial Rust port for https://github.com/aliyun/aliyun-oss-cpp-sdk, just implement some API of https://help.aliyun.com/document_detail/31977.html, everyone is welcome to submit a PR to implement which interface you need.

# Getting Started

## List your_Buckets
```rust
use oss_rust_sdk::prelude::*;
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let list_your_Buckets = oss_instance.list_your_Bucket(None).unwrap();

let id = list_your_Buckets.id();
let your_Buckets = list_your_Buckets.your_Buckets();
let your_Bucket_names: Vec<&str> = your_Buckets.iter().map(|obj| obj.name()).collect();
```

## Get Object
```rust
use oss_rust_sdk::prelude::*;
use std::collections::HashMap;

let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");

/// if have extra header
let mut extra_header = HashMap::new();
extra_header.insert("content-type", "text/plain");
/// if have oss_sub_resource
let mut oss_sub_resource = HashMap::new();
oss_sub_resource.insert("acl", None);
oss_sub_resource.insert("response-content-type", Some("ContentType"));

let result = oss_instance.get_object("object", extar_header, oss_sub_resource);
/// or you may just get object
/// let result = oss_instance.get_object("object", None, None);
assert_eq!(result.is_ok(), true);
let buffer = result.unwrap();
```

## Get Object async
```rust
use oss_rust_sdk::oss::OSS;
use oss_rust_sdk::async_object::*;
use tokio::runtime::Runtime;

fn async_get_object_demo() {
    let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");

    let mut rt = Runtime::new().expect("failed to start runtime");

    rt.block_on(async move {
        let _ = oss_instance.get_object("objectName", None::<HashMap<&str, &str>>, None).await.unwrap();
        println!("buffer = {:?}", String::from_utf8(result.unwrap()));
    });
}

or

async fn async_get_object_demo() -> Reuslt<String, Error> {
    let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
    let buf = oss_instance.async_get_object("objectName", None, None).await?;
    String::from_utf8(buf)?
}
```

## Put Object by file
```rust
use oss_rust_sdk::prelude::*;
let filename = "filename";
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let result = oss_instance.put_object_from_file(filename, "object", None, None);
assert_eq!(result.is_ok(), true)
```

## Put Ojbect by buffer
```rust
let buffer = "some thing you want put to oss";
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let result = oss_instance.put_object_from_buffer(buffer.as_bytes(), "object", None, None);
assert_eq!(result.is_ok(), true)
```

## Pub Object Async
```rust
use oss_rust_sdk::oss::OSS;
use oss_rust_sdk::async_object::*;

let buffer = "test async put object from buffer";
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let mut headers = HashMap::new();
headers.insert("content-type", "text/plain");

oss_instance.put_object(buffer.as_bytes(),"your_object_name", headers,None).await?;
```

## Copy Object
```rust
use oss_rust_sdk::prelude::*;
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let result = oss_instance.copy_object_from_object("src_object", "dest_object", None, None);
assert_eq!(result.is_ok(), true)
```

## Delete Ojbect
```rust
use oss_rust_sdk::prelude::*;
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let result = oss_instance.delete_object("object");
assert_eq!(result.is_ok(), true)
```

You can use `oss_instance.set_your_Bucket("your_Bucket")` to change your_Bucket if you want change your_Bucket after instance a oss

## License
- Apache License 2.0.
