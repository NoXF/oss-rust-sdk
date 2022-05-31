use std::collections::HashMap;

use crate::{auth::Auth, prelude::OSS, utils::to_headers};

use super::errors::{Error, ObjectError};

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::header::{HeaderMap, DATE};

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
        let object_name = object_name.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object_name, &resources_str);
        let date = self.date();
        let mut headers = if let Some(h) = headers.into() {
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
            object_name,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse().unwrap());

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
        let object_name = object_name.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), object_name, &resources_str);
        let date = self.date();

        let mut headers = if let Some(h) = headers.into() {
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
            object_name,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse().unwrap());

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
        let dest = dest.as_ref();
        let resources_str = if let Some(r) = resources.into() {
            self.get_resources_str(r)
        } else {
            String::new()
        };
        let host = self.host(self.bucket(), dest, &resources_str);
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
            dest,
            &resources_str,
            &headers,
        );
        headers.insert("Authorization", authorization.parse()?);

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
}
