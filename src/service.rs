use std::collections::HashMap;
use reqwest::header::{HeaderMap, DATE};
use failure::Error;
use quick_xml::{Reader, events::Event};

use super::oss::OSS;
use super::auth::*;

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

pub trait ServiceAPI {
    fn list_bucket(
        &self,
        resources: Option<HashMap<String, Option<String>>>,
    ) -> Result<Vec<Bucket>, Error>;
}

impl ServiceAPI for OSS {
    fn list_bucket(
        &self,
        resources: Option<HashMap<String, Option<String>>>,
    ) -> Result<Vec<Bucket>, Error> {
        let resources_str = if let Some(r) = resources {
            self.get_resources_str(r)
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

        let mut resp = self.client.get(host).headers(headers).send()?;

        let xml_str = resp.text()?;
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml_str.as_str());
        reader.trim_text(true);
        let mut buf = Vec::new();

        let mut name = String::new();
        let mut location = String::new();
        let mut create_date = String::new();
        let mut extranet_endpoint = String::new();
        let mut intranet_endpoint = String::new();
        let mut storage_class = String::new();

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"Bucket" => {
                        name = String::new();
                        location = String::new();
                        create_date = String::new();
                        extranet_endpoint = String::new();
                        intranet_endpoint = String::new();
                        storage_class = String::new();
                    }
                    b"Name" => name = reader.read_text(e.name(), &mut Vec::new())?,
                    b"CreationDate" => create_date = reader.read_text(e.name(), &mut Vec::new())?,
                    b"ExtranetEndpoint" => extranet_endpoint = reader.read_text(e.name(), &mut Vec::new())?,
                    b"IntranetEndpoint" => intranet_endpoint = reader.read_text(e.name(), &mut Vec::new())?,
                    b"Location" => location = reader.read_text(e.name(), &mut Vec::new())?,
                    b"StorageClass" => storage_class = reader.read_text(e.name(), &mut Vec::new())?,
                    _ => (),
                },
                Ok(Event::End(ref e)) if e.name() == b"Bucket" => {
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
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(result)
    }
}
