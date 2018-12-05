use std::collections::HashMap;
use reqwest::header::{DATE, HeaderMap};
use failure::Error;
use xml::reader::{EventReader, XmlEvent};

use super::oss::OSS;
use super::auth::*;


pub trait ServiceAPI {
	fn list_bucket(&self, resources: Option<HashMap<String, Option<String>>>) -> Result<Vec<String>, Error>;
}

impl ServiceAPI for OSS {
	fn list_bucket(&self, resources: Option<HashMap<String, Option<String>>>) -> Result<Vec<String>, Error> {
		let resources_str = if let Some(r) = resources {
			self.get_resources_str(r)
		} else {
			String::new()
		};
		let host = self.endpoint();
		let date = self.date();

		let mut headers = HeaderMap::new();
		headers.insert(DATE, date.parse()?);
		let authorization = self.oss_sign("GET", self.key_id(), self.key_secret(), "", "", &resources_str, &headers);
		headers.insert("Authorization", authorization.parse()?);

		let mut resp = self.client.get(host)
			.headers(headers)
			.send()?;

		let xml_str = resp.text()?;
		let mut result = Vec::new();
		let parser = EventReader::new(xml_str.as_bytes());
		let mut flag = false;
		for e in parser {
			match e {
				Ok(XmlEvent::StartElement{ name, .. }) => {
					if name.local_name == "Name" {
						flag = true;
					}
				}
				Ok(XmlEvent::Characters(name)) => {
					if flag == true {
						result.push(name);
					}
				}
				Ok(XmlEvent::EndElement{ name, .. }) => {
					if name.local_name == "Name" {
						flag = false;
					}
				}
				_ => {}
			}
		}
		Ok(result)
	}
}