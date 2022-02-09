use quick_xml::{events::Event, Reader};
use reqwest::header::{HeaderMap, CONTENT_LENGTH, DATE};
use std::collections::HashMap;

use super::auth::*;
use super::errors::{Error, ObjectError};
use super::oss::OSS;
use super::utils::*;

pub trait ObjectAPI {
    fn get_object<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<Vec<u8>, Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>;

    fn get_object_acl<S>(&self, object_name: S) -> Result<String, Error>
    where
        S: AsRef<str>;

    fn put_object_from_file<S1, S2, S3, H, R>(
        &self,
        file: S1,
        object_name: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>;

    fn put_object_from_buffer<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>;

    fn copy_object_from_object<S1, S2, S3, H, R>(
        &self,
        src: S1,
        dest: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>;

    fn delete_object<S>(&self, object_name: S) -> Result<(), Error>
    where
        S: AsRef<str>;
}

impl<'a> ObjectAPI for OSS<'a> {
    fn get_object<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<Vec<u8>, Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>,
    {
        let object_name = object_name.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object_name, &resources_str);
        let date = self.date();

        let mut headers = if let Some(h) = headers.into() {
            to_headers(h)?
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse()?);
        let authorization = self.oss_sign(
            "GET",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object_name,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let mut resp = reqwest::blocking::Client::new().get(&host).headers(headers).send()?;
        let mut buf: Vec<u8> = vec![];

        if resp.status().is_success() {
            resp.copy_to(&mut buf)?;
            Ok(buf)
        } else {
            Err(Error::Object(ObjectError::GetError {
                msg: format!("can not get object, status code: {}", resp.status()).into(),
            }))
        }
    }

    fn get_object_acl<S>(&self, object_name: S) -> Result<String, Error>
    where
        S: AsRef<str>,
    {
        let object_name = object_name.as_ref();
        let mut params: HashMap<&str, Option<&str>> = HashMap::new();
        params.insert("acl", None);
        let result = String::from_utf8(self.get_object(object_name, None, Some(params))?)?;
        let mut reader = Reader::from_str(&result);
        reader.trim_text(true);
        let mut buf = Vec::new();
        let mut grant = String::new();

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) if e.name() == b"Grant" => {
                    grant = reader.read_text(e.name(), &mut Vec::new())?;
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
        }

        Ok(grant)
    }

    fn put_object_from_file<S1, S2, S3, H, R>(
        &self,
        file: S1,
        object_name: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>,
    {
        let object_name = object_name.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object_name, &resources_str);
        let date = self.date();
        let buf = load_file(file)?;
        let mut headers = if let Some(h) = headers.into() {
            to_headers(h)?
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse()?);
        headers.insert(CONTENT_LENGTH, buf.len().to_string().parse()?);
        let authorization = self.oss_sign(
            "PUT",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object_name,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let resp = reqwest::blocking::Client::new().put(&host).headers(headers).body(buf).send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::PutError {
                msg: format!("can not put object, status code: {}", resp.status()).into(),
            }))
        }
    }

    fn put_object_from_buffer<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        H: Into<Option<HashMap<S2, S2>>>,
        R: Into<Option<HashMap<S2, Option<S2>>>>,
    {
        let object_name = object_name.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object_name, &resources_str);
        let date = self.date();
        let mut headers = if let Some(h) = headers.into() {
            to_headers(h)?
        } else {
            HeaderMap::new()
        };
        headers.insert(DATE, date.parse()?);
        headers.insert(CONTENT_LENGTH, buf.len().to_string().parse()?);
        let authorization = self.oss_sign(
            "PUT",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object_name,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let resp = reqwest::blocking::Client::new()
            .put(&host)
            .headers(headers)
            .body(buf.to_owned())
            .send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::PutError {
                msg: format!("can not put object, status code: {}", resp.status()).into(),
            }))
        }
    }

    fn copy_object_from_object<S1, S2, S3, H, R>(
        &self,
        src: S1,
        object_name: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        S3: AsRef<str>,
        H: Into<Option<HashMap<S3, S3>>>,
        R: Into<Option<HashMap<S3, Option<S3>>>>,
    {
        let object_name = object_name.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object_name, &resources_str);
        let date = self.date();
        let mut headers = if let Some(h) = headers.into() {
            to_headers(h)?
        } else {
            HeaderMap::new()
        };
        headers.insert("x-oss-copy-source", src.as_ref().parse()?);
        headers.insert(DATE, date.parse()?);
        let authorization = self.oss_sign(
            "PUT",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object_name,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let resp = reqwest::blocking::Client::new().put(&host).headers(headers).send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::CopyError {
                msg: format!("can not copy object, status code: {}", resp.status()).into(),
            }))
        }
    }

    fn delete_object<S>(&self, object_name: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        let object_name = object_name.as_ref();
        let host = self.host(self.bucket(), object_name, "");
        let date = self.date();

        let mut headers = HeaderMap::new();
        headers.insert(DATE, date.parse()?);
        let authorization = self.oss_sign(
            "DELETE",
            self.key_id(),
            self.key_secret(),
            self.bucket(),
            object_name,
            "",
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

        let resp = reqwest::blocking::Client::new().delete(&host).headers(headers).send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!("can not delete object, status code: {}", resp.status()).into(),
            }))
        }
    }
}
