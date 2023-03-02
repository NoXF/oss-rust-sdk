use async_trait::async_trait;
use quick_xml::{events::Event, Reader};
use reqwest::header::{HeaderMap, DATE};
use std::collections::HashMap;

use super::auth::*;
use super::errors::Error;
use super::oss::OSS;

#[derive(Clone, Debug)]
pub struct ListBuckets {
    prefix: String,
    marker: String,
    max_keys: String,
    is_truncated: bool,
    next_marker: String,

    id: String,
    display_name: String,

    buckets: Vec<Bucket>,
}

impl ListBuckets {
    pub fn new(
        prefix: String,
        marker: String,
        max_keys: String,
        is_truncated: bool,
        next_marker: String,
        id: String,
        display_name: String,
        buckets: Vec<Bucket>,
    ) -> Self {
        ListBuckets {
            prefix,
            marker,
            max_keys,
            is_truncated,
            next_marker,
            id,
            display_name,
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
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn buckets(&self) -> &Vec<Bucket> {
        &self.buckets
    }
}

#[derive(Clone, Debug)]
pub struct Bucket {
    name: String,
    create_date: String,
    location: String,
    extranet_endpoint: String,
    intranet_endpoint: String,
    storage_class: String,
}

impl Bucket {
    pub fn new(
        name: String,
        create_date: String,
        location: String,
        extranet_endpoint: String,
        intranet_endpoint: String,
        storage_class: String,
    ) -> Self {
        Bucket {
            name,
            create_date,
            location,
            extranet_endpoint,
            intranet_endpoint,
            storage_class,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn create_data(&self) -> &str {
        &self.create_date
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

#[async_trait]
pub trait ServiceAPI {
    async fn list_bucket<S, R>(&self, resources: R) -> Result<ListBuckets, Error>
    where
        S: AsRef<str> + Send,
        R: Into<Option<HashMap<S, Option<S>>>> + Send;
}

#[async_trait]
impl<'a> ServiceAPI for OSS<'a> {
    async fn list_bucket<S, R>(&self, resources: R) -> Result<ListBuckets, Error>
    where
        S: AsRef<str> + Send,
        R: Into<Option<HashMap<S, Option<S>>>> + Send,
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

        let resp = self.http_client.get(host).headers(headers).send().await?;

        let xml_str = resp.text().await?;
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml_str.as_str());
        reader.trim_text(true);

        let mut prefix = String::new();
        let mut marker = String::new();
        let mut max_keys = String::new();
        let mut is_truncated = false;
        let mut next_marker = String::new();
        let mut id = String::new();
        let mut display_name = String::new();

        let mut name = String::new();
        let mut location = String::new();
        let mut create_date = String::new();
        let mut extranet_endpoint = String::new();
        let mut intranet_endpoint = String::new();
        let mut storage_class = String::new();

        let list_buckets;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"Prefix" => prefix = reader.read_text(e.name())?.to_string(),
                    b"Marker" => marker = reader.read_text(e.name())?.to_string(),
                    b"MaxKeys" => max_keys = reader.read_text(e.name())?.to_string(),
                    b"IsTruncated" => {
                        is_truncated = reader.read_text(e.name())?.to_string() == "true"
                    }
                    b"NextMarker" => next_marker = reader.read_text(e.name())?.to_string(),
                    b"ID" => id = reader.read_text(e.name())?.to_string(),
                    b"DisplayName" => display_name = reader.read_text(e.name())?.to_string(),

                    b"Bucket" => {
                        name = String::new();
                        location = String::new();
                        create_date = String::new();
                        extranet_endpoint = String::new();
                        intranet_endpoint = String::new();
                        storage_class = String::new();
                    }

                    b"Name" => name = reader.read_text(e.name())?.to_string(),
                    b"CreationDate" => create_date = reader.read_text(e.name())?.to_string(),
                    b"ExtranetEndpoint" => {
                        extranet_endpoint = reader.read_text(e.name())?.to_string()
                    }
                    b"IntranetEndpoint" => {
                        intranet_endpoint = reader.read_text(e.name())?.to_string()
                    }
                    b"Location" => location = reader.read_text(e.name())?.to_string(),
                    b"StorageClass" => storage_class = reader.read_text(e.name())?.to_string(),
                    _ => (),
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == b"Bucket" => {
                    let bucket = Bucket::new(
                        name.clone(),
                        create_date.clone(),
                        location.clone(),
                        extranet_endpoint.clone(),
                        intranet_endpoint.clone(),
                        storage_class.clone(),
                    );
                    result.push(bucket);
                }
                Ok(Event::Eof) => {
                    list_buckets = ListBuckets::new(
                        prefix,
                        marker,
                        max_keys,
                        is_truncated,
                        next_marker,
                        id,
                        display_name,
                        result,
                    );
                    break;
                } // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
        }
        Ok(list_buckets)
    }
}
