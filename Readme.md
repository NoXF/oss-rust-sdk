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

// if have extra header
let mut extra_header = HashMap::new();
extra_header.insert("content-type", "text/plain");
// if have oss_sub_resource
let oss_sub_resource = HashMap::new();
oss_sub_resource.insert("acl", None);
oss_sub_resource.insert("response-content-type", Some("ContentType"))

let result = oss_instance.get_object("object", Some(extar_header), Some(oss_sub_resource));
// or you may just get object
// let result = oss_instance.get_object("object", None, None);
assert_eq!(result.is_ok(), true)
let buffer = result.unwrap().text();
```

## Get Object async
```rust
fn wrap() -> impl Future<Item=String, Error=(Error)> {
    let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
    let ret = oss_instance.async_get_object("your_object_name", None, None)
        .map(|body| {
            str::from_utf8(&body).unwrap().to_string()
        });
    ret
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
let buffer = "test async put object from buffer";
let oss_instance = OSS::new("your_AccessKeyId", "your_AccessKeySecret", "your_Endpoint", "your_Bucket");
let mut headers = HashMap::new();
headers.insert("content-type", "text/plain");

// return Future<Item = (async_reqwest::Response), Error = Error>
oss_instance.async_put_object_from_buffer(buffer.as_bytes(),"your_object_name",Some(headers),None);
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

## TODO:

- complete object api
- complete your_Bucket api
- support async api

## License
- Apache License 2.0.