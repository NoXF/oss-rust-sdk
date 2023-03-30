use reqwest::header::{HeaderMap, DATE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::prelude::Owner;

use super::auth::*;
use super::errors::Error;
use super::oss::OSS;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListBuckets {
    #[serde(default)]
    prefix: String,
    #[serde(default)]
    marker: String,
    #[serde(default)]
    max_keys: String,
    #[serde(default)]
    is_truncated: bool,
    #[serde(default)]
    next_marker: String,

    owner: Owner,

    #[serde(default)]
    buckets: Vec<Bucket>,
}

impl ListBuckets {
    pub fn new(
        prefix: String,
        marker: String,
        max_keys: String,
        is_truncated: bool,
        next_marker: String,
        owner: Owner,
        buckets: Vec<Bucket>,
    ) -> Self {
        ListBuckets {
            prefix,
            marker,
            max_keys,
            is_truncated,
            next_marker,
            owner,
            buckets,
        }
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn marker(&self) -> &str {
        &self.marker
    }

    pub fn max_keys(&self) -> &str {
        &self.max_keys
    }

    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }

    pub fn next_marker(&self) -> &str {
        &self.next_marker
    }

    pub fn id(&self) -> &str {
        &self.owner.id
    }

    pub fn display_name(&self) -> &str {
        &self.owner.display_name
    }

    pub fn buckets(&self) -> &Vec<Bucket> {
        &self.buckets
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bucket {
    #[serde(default)]
    name: String,
    #[serde(default)]
    creation_date: String,
    #[serde(default)]
    location: String,
    #[serde(default)]
    extranet_endpoint: String,
    #[serde(default)]
    intranet_endpoint: String,
    #[serde(default)]
    storage_class: String,
}

impl Bucket {
    pub fn new(
        name: String,
        creation_date: String,
        location: String,
        extranet_endpoint: String,
        intranet_endpoint: String,
        storage_class: String,
    ) -> Self {
        Bucket {
            name,
            creation_date,
            location,
            extranet_endpoint,
            intranet_endpoint,
            storage_class,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn creation_date(&self) -> &str {
        &self.creation_date
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn extranet_endpoint(&self) -> &str {
        &self.extranet_endpoint
    }

    pub fn intranet_endpoint(&self) -> &str {
        &self.intranet_endpoint
    }

    pub fn storage_class(&self) -> &str {
        &self.storage_class
    }
}

pub trait ServiceAPI {
    fn list_bucket<S, R>(&self, resources: R) -> Result<ListBuckets, Error>
    where
        S: AsRef<str>,
        R: Into<Option<HashMap<S, Option<S>>>>;
}

impl<'a> ServiceAPI for OSS<'a> {
    fn list_bucket<S, R>(&self, resources: R) -> Result<ListBuckets, Error>
    where
        S: AsRef<str>,
        R: Into<Option<HashMap<S, Option<S>>>>,
    {
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(&r)
        } else {
            String::new()
        };
        let host = self.endpoint();
        let date = self.date();

        let mut headers = HeaderMap::new();
        headers.insert(DATE, date.parse()?);
        let authorization = self.oss_sign(
            "GET",
            self.key_id(),
            self.key_secret(),
            "",
            "",
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let resp = reqwest::blocking::Client::new()
            .get(host)
            .headers(headers)
            .send()?;

        let body = resp.text()?;
        let list_buckets = quick_xml::de::from_str::<ListBuckets>(&body)?;

        Ok(list_buckets)
    }
}
