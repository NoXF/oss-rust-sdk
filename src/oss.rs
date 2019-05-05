use std::{mem, str};
use std::collections::HashMap;
use std::borrow::Cow;
use chrono::prelude::*;
use reqwest::Client;
use reqwest::r#async as async_reqwest;
use reqwest::header::{HeaderMap, DATE};
use futures::{Future, Stream};
pub use reqwest::r#async::Chunk;

use failure::Error;
use super::auth::*;
use super::utils::*;

#[derive(Clone, Debug)]
pub struct OSS<'a> {
    key_id: Cow<'a, str>,
    key_secret: Cow<'a, str>,
    endpoint: Cow<'a, str>,
    bucket: Cow<'a, str>,
    pub client: Client,
}

const RESOURCES: [&str; 50] = [
    "acl",
    "uploads",
    "location",
    "cors",
    "logging",
    "website",
    "referer",
    "lifecycle",
    "delete",
    "append",
    "tagging",
    "objectMeta",
    "uploadId",
    "partNumber",
    "security-token",
    "position",
    "img",
    "style",
    "styleName",
    "replication",
    "replicationProgress",
    "replicationLocation",
    "cname",
    "bucketInfo",
    "comp",
    "qos",
    "live",
    "status",
    "vod",
    "startTime",
    "endTime",
    "symlink",
    "x-oss-process",
    "response-content-type",
    "response-content-language",
    "response-expires",
    "response-cache-control",
    "response-content-disposition",
    "response-content-encoding",
    "udf",
    "udfName",
    "udfImage",
    "udfId",
    "udfImageDesc",
    "udfApplication",
    "comp",
    "udfApplicationLog",
    "restore",
    "callback",
    "callback-var",
];

impl<'a> OSS<'a> {
    pub fn new<S> (key_id: S, key_secret: S, endpoint: S, bucket: S) -> Self
        where S: Into<Cow<'a, str>> {
        OSS {
            key_id: key_id.into(),
            key_secret: key_secret.into(),
            endpoint: endpoint.into(),
            bucket: bucket.into(),
            client: Client::new(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn key_secret(&self) -> &str {
        &self.key_secret
    }

    pub fn set_bucket(&mut self, bucket: &'a str) {
        self.bucket = bucket.into()
    }

    pub fn host(&self, bucket: &str, object: &str, resources_str: &str) -> String {
        if self.endpoint.starts_with("https") {
            format!(
                "https://{}.{}/{}?{}",
                bucket,
                self.endpoint.replacen("https://", "", 1),
                object,
                resources_str
            )
        } else {
            format!(
                "http://{}.{}/{}?{}",
                bucket,
                self.endpoint.replacen("http://", "", 1),
                object,
                resources_str
            )
        }
    }

    pub fn date(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%a, %d %b %Y %T GMT").to_string()
    }

    pub fn get_resources_str(&self, params: HashMap<String, Option<String>>) -> String {
        let mut resources: Vec<(&String, &Option<String>)> = params
            .iter()
            .filter(|(k, _)| RESOURCES.contains(&k.as_str()))
            .collect();
        resources.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));
        let mut result = String::new();
        for (k, v) in resources {
            if !result.is_empty() {
                result += "&";
            }
            if let Some(vv) = v {
                result += &format!("{}={}", k.to_owned(), vv);
            } else {
                result += &k;
            }
        }
        result
    }

    pub fn async_get_object(
        &self,
        object: &str,
        headers: Option<HashMap<&str, &str>>,
        resources: Option<HashMap<String, Option<String>>>,
    ) -> impl Future<Item = Chunk, Error = Error> {
        let resources_str = if let Some(r) = resources {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object, &resources_str);
        let date = self.date();
        let mut headers = if let Some(h) = headers {
            to_headers(h).unwrap()
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse().unwrap());
        let authorization = self.oss_sign(
            "GET",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse().unwrap());

        async_reqwest::Client::new()
            .get(&host)
            .headers(headers)
            .send()
            .and_then(|mut resp| {
                let body = mem::replace(resp.body_mut(), async_reqwest::Decoder::empty());
                body.concat2()
            })
            .map_err(Error::from)
    }

    pub fn async_put_object_from_buffer(
        &self,
        buf: &[u8],
        object: &str,
        headers: Option<HashMap<&str, &str>>,
        resources: Option<HashMap<String, Option<String>>>,
    ) -> impl Future<Item = (async_reqwest::Response), Error = Error> {
        let resources_str = if let Some(r) = resources {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object, &resources_str);
        let date = self.date();

        let mut headers = if let Some(h) = headers {
            to_headers(h).unwrap()
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse().unwrap());
        let authorization = self.oss_sign(
            "PUT",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse().unwrap());

        async_reqwest::Client::new()
            .put(&host)
            .headers(headers)
            .body(buf.to_owned())
            .send()
            .map(|resp| resp)
            .map_err(Error::from)
    }
}
