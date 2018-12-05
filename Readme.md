# Getting Started

## Get Object
```rust
use std::collections::HashMap;

let oss_instance = OSS::new("AccessKeyId", "AccessKeySecret", "Endpoint", "Bucket");
let mut extra_header = HashMap::new();
extra_header.insert("content-type", "text/plain");
let oss_sub_resource = HashMap::new();
oss_sub_resource.insert("acl", None);
oss_sub_resource.insert("response-content-type", Some("ContentType"))
let result = oss_instance.get_object("object", Some(extar_header), Some(oss_sub_resource));
// or you may just get object
// let result = oss_instance.get_object("object", None, None);
assert_eq!(result.is_ok(), true)
let buffer = result.unwrap().text();
```

## Put Object by file
```rust
let filename = "filename";
let oss_instance = OSS::new("AccessKeyId", "AccessKeySecret", "Endpoint", "Bucket");
let result = oss_instance.put_object_from_file(filename, "object", None, None);
assert_eq!(result.is_ok(), true)
```

## Put Ojbect by buffer
```rust
let buffer = "some thing you want put to oss";
let oss_instance = OSS::new("AccessKeyId", "AccessKeySecret", "Endpoint", "Bucket");
let result = oss_instance.put_object_from_buffer(buffer.as_bytes(), "object", None, None);
assert_eq!(result.is_ok(), true)
```

## Copy Object
```rust
let oss_instance = OSS::new("AccessKeyId", "AccessKeySecret", "Endpoint", "Bucket");
let result = oss_instance.copy_object_from_object("src_object", "dest_object", None, None);
assert_eq!(result.is_ok(), true)
```

## Delete Ojbect
```rust
let oss_instance = OSS::new("AccessKeyId", "AccessKeySecret", "Endpoint", "Bucket");
let result = oss_instance.delete_object("object");
assert_eq!(result.is_ok(), true)
```

You can use `oss_instance.set_bucket("Bucket")` to change bucket if you want change bucket after instance a oss

## TODO:

- complete object api
- complete bucket api
- support async api

## License
- Apache License 2.0.