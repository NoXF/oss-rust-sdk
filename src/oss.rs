use chrono::prelude::*;
use reqwest::header::{HeaderMap, DATE};
use reqwest::Client;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str;
use std::time::{Duration, SystemTime};

use crate::errors::ObjectError;

use super::auth::*;
use super::errors::Error;
use super::utils::*;

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

#[derive(Clone, Debug)]
pub struct OSS<'a> {
    key_id: Cow<'a, str>,
    key_secret: Cow<'a, str>,
    endpoint: Cow<'a, str>,
    bucket: Cow<'a, str>,

    pub(crate) http_client: Client,
}

#[derive(Default)]
pub struct Options {
    pub pool_max_idle_per_host: Option<usize>,
    pub timeout: Option<Duration>,
}

impl<'a> OSS<'a> {
    pub fn new<S>(key_id: S, key_secret: S, endpoint: S, bucket: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self::new_with_opts(key_id, key_secret, endpoint, bucket, Default::default())
    }

    pub fn new_with_opts<S>(key_id: S, key_secret: S, endpoint: S, bucket: S, opts: Options) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        let mut builder = Client::builder();
        if let Some(timeout) = opts.timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(max_per_host) = opts.pool_max_idle_per_host {
            builder = builder.pool_max_idle_per_host(max_per_host);
        }

        let http_client = builder.build().expect("Build http client failed");
        OSS {
            key_id: key_id.into(),
            key_secret: key_secret.into(),
            endpoint: endpoint.into(),
            bucket: bucket.into(),
            http_client,
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

    pub fn get_resources_str<S>(&self, params: &HashMap<S, Option<S>>) -> String
    where
        S: AsRef<str>,
    {
        let mut resources: Vec<(&S, &Option<S>)> = params
            .iter()
            .filter(|(k, _)| RESOURCES.contains(&k.as_ref()))
            .collect();
        resources.sort_by(|a, b| a.0.as_ref().to_string().cmp(&b.0.as_ref().to_string()));
        let mut result = String::new();
        for (k, v) in resources {
            if !result.is_empty() {
                result += "&";
            }
            if let Some(vv) = v {
                result += &format!("{}={}", k.as_ref().to_owned(), vv.as_ref());
            } else {
                result += k.as_ref();
            }
        }
        result
    }

    pub fn get_params_str<S>(&self, params: &HashMap<S, Option<S>>) -> String
    where
        S: AsRef<str>,
    {
        let mut resources: Vec<(&S, &Option<S>)> = params.iter().collect();
        resources.sort_by(|a, b| a.0.as_ref().to_string().cmp(&b.0.as_ref().to_string()));
        let mut result = String::new();
        for (k, v) in resources {
            if !result.is_empty() {
                result += "&";
            }
            if let Some(vv) = v {
                result += &format!("{}={}", k.as_ref().to_owned(), vv.as_ref());
            } else {
                result += k.as_ref();
            }
        }
        result
    }

    /// Build a request. Return url and header for reqwest client builder.
    pub fn build_request<S1, S2, H, R>(
        &self,
        req_type: RequestType,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(String, HeaderMap), Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>,
    {
        let object_name = object_name.as_ref();
        let (resources_str, params_str) = if let Some(r) = resources.into() {
            (self.get_resources_str(&r), self.get_params_str(&r))
        } else {
            (String::new(), String::new())
        };

        let host = self.host(self.bucket(), object_name, &params_str);
        let date = self.date();
        let mut headers = if let Some(h) = headers.into() {
            to_headers(h)?
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse()?);
        let authorization = self.oss_sign(
            req_type.as_str(),
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object_name,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        Ok((host, headers))
    }
}

pub enum RequestType {
    Get,
    Put,
    Delete,
    Head,
}

impl RequestType {
    fn as_str(&self) -> &str {
        match self {
            RequestType::Get => "GET",
            RequestType::Put => "PUT",
            RequestType::Delete => "DELETE",
            RequestType::Head => "HEAD",
        }
    }
}

#[derive(Debug)]
pub struct ObjectMeta {
    /// The last modified time
    pub last_modified: SystemTime,
    /// The size in bytes of the object
    pub size: usize,
    /// 128-bits RFC 1864 MD5. This field only presents in normal file. Multipart and append-able file will have empty md5.
    pub md5: String,
}

impl ObjectMeta {
    pub fn from_header_map(header: &HeaderMap) -> Result<Self, Error> {
        let getter = |key: &str| -> Result<&str, Error> {
            let value = header
                .get(key)
                .ok_or_else(|| {
                    Error::Object(ObjectError::HeadError {
                        msg: format!(
                            "can not find {} in head response, response header: {:?}",
                            key, header
                        )
                        .into(),
                    })
                })?
                .to_str()
                .map_err(|_| {
                    Error::Object(ObjectError::HeadError {
                        msg: format!("header entry {} contains invalid ASCII code", key).into(),
                    })
                })?;
            Ok(value)
        };

        let last_modified = httpdate::parse_http_date(getter("Last-Modified")?).map_err(|e| {
            Error::Object(ObjectError::HeadError {
                msg: format!("cannot parse to system time: {}", e).into(),
            })
        })?;
        let size = getter("Content-Length")?.parse().map_err(|e| {
            Error::Object(ObjectError::HeadError {
                msg: format!("cannot parse to number: {}", e).into(),
            })
        })?;
        let md5 = getter("Content-Md5")?.to_string();

        Ok(Self {
            last_modified,
            size,
            md5,
        })
    }
}
