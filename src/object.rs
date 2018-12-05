use reqwest::header::{DATE, CONTENT_LENGTH, HeaderMap};
use std::collections::HashMap;

use failure::Error;
use super::error::ObjectError;
use super::oss::OSS;
use super::auth::*;
use super::utils::*;

pub trait ObjectAPI {
	fn get_object(&self, object: &str, headers: Option<HashMap<&str, &str>>, resources: Option<HashMap<String, Option<String>>>) -> Result<String, Error>;
	fn put_object_from_file(&self, file: &str, object: &str, headers: Option<HashMap<&str, &str>>, resources: Option<HashMap<String, Option<String>>>) -> Result<(), Error>;
	fn put_object_from_buffer(&self, buf: &[u8], object: &str, headers: Option<HashMap<&str, &str>>, resources: Option<HashMap<String, Option<String>>>) -> Result<(), Error>;
	fn copy_object_from_object(&self, src: &str, dest: &str, headers: Option<HashMap<&str, &str>>, resources: Option<HashMap<String, Option<String>>>) -> Result<(), Error>;
	fn delete_object(&self, object: &str) -> Result<(), Error>;
}

impl ObjectAPI for OSS {
	fn get_object(&self, object: &str, headers: Option<HashMap<&str, &str>>, resources: Option<HashMap<String, Option<String>>>) -> Result<String, Error> {
		let resources_str = if let Some(r) = resources {
			self.get_resources_str(r)
		} else {
			String::new()
		};
		let host = self.host(self.bucket(), object, &resources_str);
		let date = self.date();

		let mut headers = if let Some(h) = headers { to_headers(h)? } else { HeaderMap::new() };
		headers.insert(DATE, date.parse()?);
		let authorization = self.oss_sign("GET", self.key_id(), self.key_secret(), self.bucket(), object, &resources_str, &headers);
		headers.insert("Authorization", authorization.parse()?);

		let mut resp = self.client.get(&host)
			.headers(headers)
			.send()?;

		if resp.status().is_success() {
			Ok(resp.text()?)
		} else {
			Err(ObjectError::GetError(format!("can not get object, status code: {}", resp.status())).into())
		}
	}

	fn put_object_from_file(&self, file: &str, object: &str, headers: Option<HashMap<&str, &str>>, resources: Option<HashMap<String, Option<String>>>) -> Result<(), Error> {
		let resources_str = if let Some(r) = resources {
			self.get_resources_str(r)
		} else {
			String::new()
		};
		let host = self.host(self.bucket(), object, &resources_str);
		let date = self.date();
		let buf = load_file_to_string(file)?;
		let mut headers = if let Some(h) = headers { to_headers(h)? } else { HeaderMap::new() };
		headers.insert(DATE, date.parse()?);
		headers.insert(CONTENT_LENGTH, buf.len().to_string().parse()?);
		let authorization = self.oss_sign("PUT", self.key_id(), self.key_secret(), self.bucket(), object, &resources_str, &headers);
		headers.insert("Authorization", authorization.parse()?);

		let resp = self.client.put(&host)
			.headers(headers)
			.body(buf)
			.send()?;

		if resp.status().is_success() {
			Ok(())
		} else {
			Err(ObjectError::PutError(format!("could not put object to oss, status code: {}", resp.status())).into())
		}
	}

	fn put_object_from_buffer(&self, buf: &[u8], object: &str, headers: Option<HashMap<&str, &str>>, resources: Option<HashMap<String, Option<String>>>) -> Result<(), Error> {
		let resources_str = if let Some(r) = resources {
			self.get_resources_str(r)
		} else {
			String::new()
		};
		let host = self.host(self.bucket(), object, &resources_str);
		let date = self.date();
		let mut headers = if let Some(h) = headers { to_headers(h)? } else { HeaderMap::new() };
		headers.insert(DATE, date.parse()?);
		headers.insert(CONTENT_LENGTH, buf.len().to_string().parse()?);
		let authorization = self.oss_sign("PUT", self.key_id(), self.key_secret(), self.bucket(), object, &resources_str, &headers);
		headers.insert("Authorization", authorization.parse()?);

		let resp = self.client.put(&host)
			.headers(headers)
			.body(buf.to_owned())
			.send()?;

		if resp.status().is_success() {
			Ok(())
		} else {
			Err(ObjectError::PutError(format!("could not put object to oss, status code: {}", resp.status())).into())
		}
	}

	fn copy_object_from_object(&self, src: &str, object: &str, headers: Option<HashMap<&str, &str>>, resources: Option<HashMap<String, Option<String>>>) -> Result<(), Error> {
		let resources_str = if let Some(r) = resources {
			self.get_resources_str(r)
		} else {
			String::new()
		};
		let host = self.host(self.bucket(), object, &resources_str);
		let date = self.date();
		let mut headers = if let Some(h) = headers { to_headers(h)? } else { HeaderMap::new() };
		headers.insert("x-oss-copy-source", src.parse()?);
		headers.insert(DATE, date.parse()?);
		let authorization = self.oss_sign("PUT", self.key_id(), self.key_secret(), self.bucket(), object, &resources_str, &headers);
		headers.insert("Authorization", authorization.parse()?);

		let resp = self.client.put(&host)
			.headers(headers)
			.send()?;

		if resp.status().is_success() {
			Ok(())
		} else {
			Err(ObjectError::CopyError(format!("could not copy object, status code: {}", resp.status())).into())
		}
	}

	fn delete_object(&self, object: &str) -> Result<(), Error> {
		let host = self.host(self.bucket(), object, "");
		let date = self.date();

		let mut headers = HeaderMap::new();
		headers.insert(DATE, date.parse()?);
		let authorization = self.oss_sign("DELETE", self.key_id(), self.key_secret(), self.bucket(), object, "", &headers);
		headers.insert("Authorization", authorization.parse()?);

		let resp = self.client.delete(&host)
			.headers(headers)
			.send()?;

		if resp.status().is_success() {
			Ok(())
		} else {
			Err(ObjectError::DeleteError(format!("could not delete object, status code: {}", resp.status())).into())
		}
	}
}