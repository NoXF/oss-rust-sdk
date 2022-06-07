use std::collections::HashMap;

use crate::{
    oss::{ObjectMeta, RequestType},
    prelude::OSS,
};

use super::errors::{Error, ObjectError};

use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait AsyncObjectAPI {
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

        let resp = reqwest::Client::new()
            .get(&host)
            .headers(headers)
            .send()
            .await?;

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

        let resp = reqwest::Client::new()
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

        let resp = reqwest::Client::new()
            .put(&host)
            .headers(headers)
            .send()
            .await?;

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

        let resp = reqwest::Client::new()
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

        let resp = reqwest::Client::new()
            .head(&host)
            .headers(headers)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(ObjectMeta::from_header_map(resp.headers())?)
        } else {
            Err(Error::Object(ObjectError::DeleteError {
                msg: format!("can not head object, status code: {}", resp.status()).into(),
            }))
        }
    }
}
