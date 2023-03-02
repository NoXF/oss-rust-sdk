use quick_xml::{events::Event, Reader};
use std::collections::HashMap;

use crate::{
    oss::{ObjectMeta, RequestType},
    prelude::{ListObjects, Object, OSS},
};

use super::errors::{Error, ObjectError};

use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait AsyncObjectAPI {
    async fn list_object<S, H, R>(&self, headers: H, resources: R) -> Result<ListObjects, Error>
    where
        S: AsRef<str>,
        H: Into<Option<HashMap<S, S>>> + Send,
        R: Into<Option<HashMap<S, Option<S>>>> + Send;

    async fn get_object<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<Bytes, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send;

    async fn put_object<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send;

    async fn copy_object_from_object<S1, S2, S3, H, R>(
        &self,
        src: S1,
        dest: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        S3: AsRef<str> + Send,
        H: Into<Option<HashMap<S3, S3>>> + Send,
        R: Into<Option<HashMap<S3, Option<S3>>>> + Send;

    async fn delete_object<S>(&self, object_name: S) -> Result<(), Error>
    where
        S: AsRef<str> + Send;

    async fn head_object<S>(&self, object_name: S) -> Result<ObjectMeta, Error>
    where
        S: AsRef<str> + Send;
}

#[async_trait]
impl<'a> AsyncObjectAPI for OSS<'a> {
    async fn list_object<S, H, R>(&self, headers: H, resources: R) -> Result<ListObjects, Error>
    where
        S: AsRef<str>,
        H: Into<Option<HashMap<S, S>>> + Send,
        R: Into<Option<HashMap<S, Option<S>>>> + Send,
    {
        let (host, headers) =
            self.build_request(RequestType::Get, String::new(), headers, resources)?;

        let resp = self.http_client.get(host).headers(headers).send().await?;

        let xml_str = resp.text().await?;
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml_str.as_str());
        reader.trim_text(true);

        let mut bucket_name = String::new();
        let mut prefix = String::new();
        let mut marker = String::new();
        let mut max_keys = String::new();
        let mut delimiter = String::new();
        let mut is_truncated = false;

        let mut key = String::new();
        let mut last_modified = String::new();
        let mut etag = String::new();
        let mut r#type = String::new();
        let mut size = 0usize;
        let mut storage_class = String::new();
        let mut owner_id = String::new();
        let mut owner_display_name = String::new();

        let list_objects;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"Name" => bucket_name = reader.read_text(e.name())?.to_string(),
                    b"Prefix" => prefix = reader.read_text(e.name())?.to_string(),
                    b"Marker" => marker = reader.read_text(e.name())?.to_string(),
                    b"MaxKeys" => max_keys = reader.read_text(e.name())?.to_string(),
                    b"Delimiter" => delimiter = reader.read_text(e.name())?.to_string(),
                    b"IsTruncated" => {
                        is_truncated = reader.read_text(e.name())?.to_string() == "true"
                    }
                    b"Contents" => {
                        // do nothing
                    }
                    b"Key" => key = reader.read_text(e.name())?.to_string(),
                    b"LastModified" => last_modified = reader.read_text(e.name())?.to_string(),
                    b"ETag" => etag = reader.read_text(e.name())?.to_string(),
                    b"Type" => r#type = reader.read_text(e.name())?.to_string(),
                    b"Size" => size = reader.read_text(e.name())?.parse::<usize>().unwrap(),
                    b"StorageClass" => storage_class = reader.read_text(e.name())?.to_string(),
                    b"Owner" => {
                        // do nothing
                    }
                    b"ID" => owner_id = reader.read_text(e.name())?.to_string(),
                    b"DisplayName" => owner_display_name = reader.read_text(e.name())?.to_string(),

                    _ => (),
                },

                Ok(Event::End(ref e)) if e.name().as_ref() == b"Contents" => {
                    let object = Object::new(
                        key.clone(),
                        last_modified.clone(),
                        size,
                        etag.clone(),
                        r#type.clone(),
                        storage_class.clone(),
                        owner_id.clone(),
                        owner_display_name.clone(),
                    );
                    result.push(object);
                }
                Ok(Event::Eof) => {
                    list_objects = ListObjects::new(
                        bucket_name,
                        delimiter,
                        prefix,
                        marker,
                        max_keys,
                        is_truncated,
                        result,
                    );
                    break;
                } // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }
        }
        Ok(list_objects)
    }
    async fn get_object<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<Bytes, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send,
    {
        let (host, headers) =
            self.build_request(RequestType::Get, object_name, headers, resources)?;

        let resp = self.http_client.get(&host).headers(headers).send().await?;

        if resp.status().is_success() {
            Ok(resp.bytes().await?)
        } else {
            Err(Error::Object(ObjectError::GetError {
                msg: format!("can not get object, status code: {}", resp.status()).into(),
            }))
        }
    }

    async fn put_object<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send,
    {
        let (host, headers) =
            self.build_request(RequestType::Put, object_name, headers, resources)?;

        let resp = self
            .http_client
            .put(&host)
            .headers(headers)
            .body(buf.to_owned())
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!(
                    "can not put object, status code, status code: {}",
                    resp.status()
                )
                .into(),
            }))
        }
    }

    async fn copy_object_from_object<S1, S2, S3, H, R>(
        &self,
        src: S1,
        dest: S2,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        S3: AsRef<str> + Send,
        H: Into<Option<HashMap<S3, S3>>> + Send,
        R: Into<Option<HashMap<S3, Option<S3>>>> + Send,
    {
        let (host, mut headers) = self.build_request(RequestType::Put, dest, headers, resources)?;
        headers.insert("x-oss-copy-source", src.as_ref().parse()?);

        let resp = self.http_client.put(&host).headers(headers).send().await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::CopyError {
                msg: format!("can not copy object, status code: {}", resp.status()).into(),
            }))
        }
    }

    async fn delete_object<S>(&self, object_name: S) -> Result<(), Error>
    where
        S: AsRef<str> + Send,
    {
        let headers = HashMap::<String, String>::new();
        let (host, headers) =
            self.build_request(RequestType::Delete, object_name, Some(headers), None)?;

        let resp = self
            .http_client
            .delete(&host)
            .headers(headers)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!("can not delete object, status code: {}", resp.status()).into(),
            }))
        }
    }

    async fn head_object<S>(&self, object_name: S) -> Result<ObjectMeta, Error>
    where
        S: AsRef<str> + Send,
    {
        let (host, headers) = self.build_request(
            RequestType::Head,
            object_name,
            None::<HashMap<String, String>>,
            None,
        )?;

        let resp = self.http_client.head(&host).headers(headers).send().await?;

        if resp.status().is_success() {
            Ok(ObjectMeta::from_header_map(resp.headers())?)
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!("can not head object, status code: {}", resp.status()).into(),
            }))
        }
    }
}
