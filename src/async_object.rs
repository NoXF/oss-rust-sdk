use std::collections::HashMap;

use crate::{
    multi_part::{CompleteMultipartUploadResult, InitiateMultipartUploadResult},
    oss::{ObjectMeta, RequestType},
    prelude::{ListObjects, OSS},
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

    /// Notify oss to init a Multipart Upload event
    async fn init_multi<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<InitiateMultipartUploadResult, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send;

    /// Upload data in chunks according to the specified Object name and uploadId
    async fn upload_part<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<String, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send;

    /// Complete the multipart upload of the entire file
    ///
    /// body format
    /// <CompleteMultipartUpload>
    /// <Part>
    /// <PartNumber>PartNumber</PartNumber>
    /// <ETag>ETag</ETag>
    /// </Part>
    /// ...
    /// </CompleteMultipartUpload>
    ///
    /// # Examples
    ///
    ///  #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    ///  #[serde(rename_all = "PascalCase")]
    ///  pub struct PartWrapper {
    ///      pub part: Vec<Part>,
    ///  }
    ///
    ///  #[derive(Debug, Serialize, Deserialize, PartialEq)]
    ///  #[serde(rename_all = "PascalCase")]
    ///  pub struct Part {
    ///      part_number: usize,
    ///      e_tag: String,
    ///  }
    ///
    ///  let parts = CompleteDTO {
    ///      part: vec![Part {
    ///          part_number: 1,
    ///          e_tag: "50BE5FACC702C5B945588031C6*****".to_string(),
    ///      }],
    ///  };
    ///
    ///  let body = quick_xml::se::to_string_with_root("CompleteMultipartUpload", &parts).unwrap();
    ///
    async fn complete_multi<S1, S2, H, R>(
        &self,
        body: String,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<CompleteMultipartUploadResult, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send;

    /// Cancel the MultipartUpload event and delete the corresponding Part data
    async fn abort_multi<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<(), Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send;
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
        let body = resp.text().await?;
        let list_objects = quick_xml::de::from_str::<ListObjects>(&body)?;

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

    async fn init_multi<S1, S2, H, R>(
        &self,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<InitiateMultipartUploadResult, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send,
    {
        let (host, headers) =
            self.build_request(RequestType::Post, object_name, headers, resources)?;

        let resp = self.http_client.post(&host).headers(headers).send().await?;

        if resp.status().is_success() {
            let body = resp.text().await?;
            let res = quick_xml::de::from_str::<InitiateMultipartUploadResult>(&body)?;
            Ok(res)
        } else {
            Err(Error::Object(ObjectError::PostError {
                msg: format!(
                    "init multi failed, status code, status code: {}",
                    resp.status()
                )
                .into(),
            }))
        }
    }

    async fn upload_part<S1, S2, H, R>(
        &self,
        buf: &[u8],
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<String, Error>
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
            let e_tag = resp.headers().get("ETag").unwrap().to_str().unwrap();
            Ok(e_tag.to_string())
        } else {
            Err(Error::Object(ObjectError::PutError {
                msg: format!(
                    "can not put object, status code, status code: {}",
                    resp.status()
                )
                .into(),
            }))
        }
    }

    async fn complete_multi<S1, S2, H, R>(
        &self,
        body: String,
        object_name: S1,
        headers: H,
        resources: R,
    ) -> Result<CompleteMultipartUploadResult, Error>
    where
        S1: AsRef<str> + Send,
        S2: AsRef<str> + Send,
        H: Into<Option<HashMap<S2, S2>>> + Send,
        R: Into<Option<HashMap<S2, Option<S2>>>> + Send,
    {
        let (host, headers) =
            self.build_request(RequestType::Post, object_name, headers, resources)?;

        let resp = self
            .http_client
            .post(&host)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        if resp.status().is_success() {
            let body = resp.text().await?;
            let res = quick_xml::de::from_str::<CompleteMultipartUploadResult>(&body)?;
            Ok(res)
        } else {
            Err(Error::Object(ObjectError::PostError {
                msg: format!(
                    "complete multi failed, status code, status code: {}",
                    resp.status()
                )
                .into(),
            }))
        }
    }

    async fn abort_multi<S1, S2, H, R>(
        &self,
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
            self.build_request(RequestType::Delete, object_name, headers, resources)?;

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
                msg: format!(
                    "abort multi failed, status code, status code: {}",
                    resp.status()
                )
                .into(),
            }))
        }
    }
}
