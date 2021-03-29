/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use once_cell::sync::OnceCell;
use serde::{Deserialize, de::DeserializeOwned};
use isahc::{HttpClientBuilder, HttpClient, AsyncReadResponseExt};
use isahc::config::{Configurable, RedirectPolicy};
use async_trait::async_trait;
use futures::future;

use crate::esr_errors::Result;

fn get_static_client() -> Result<&'static HttpClient> {
    static RET: OnceCell<HttpClient> = OnceCell::new();
    let init = || HttpClientBuilder::new()
        //.connect_timeout(Duration::from_secs(5))
        .max_connections(32)
        .redirect_policy(RedirectPolicy::Limit(5))
        .auto_referer()
        .default_header("User-Agent", "cargo-esr/0.1")
        .build();
    let ret = RET.get_or_try_init(init)?;
    Ok(ret)

}

#[derive(Deserialize, Debug, Clone)]
pub struct Meta {
    pub total: usize,
}

#[async_trait]
pub trait EsrFromMulti: EsrFrom + Sync + Send + 'static {
    type Inner: Clone;

    async fn from_url_multi(url: &str, multi_page: bool) -> Result<Self> {
        let mut initial_self = Self::from_url(url).await?;
        let total = initial_self.total_from_meta();

        // per_page=100 is the maximum number allowed.
        // If total > 100, GET all pages and append all results to initial_self inner vec
        if multi_page && total > 100 {
            let num_pages = (total as f64 / 100.0).ceil() as usize;

            let more_pages_iter = (2..=num_pages)
                .map(|page| url.to_owned() + &*format!("&page={}", page))
                .map(|page_url| Self::from_url_owned(page_url));


            for page_res in future::join_all(more_pages_iter).await {
                let page = page_res?;
                initial_self.get_inner_mut().extend_from_slice(&*page.get_inner());
            }

        }

        if multi_page && initial_self.get_inner().len() != total {
            Err("Total no. of results does not match total reported")?;
        }

        Ok(initial_self)
    }

    // Owned arguments variants to allow use in task::spawn
    async fn from_url_multi_owned(url: String, multi_page: bool) -> Result<Self> {
        Self::from_url_multi(&*url, multi_page).await
    }

    fn total_from_meta(&self) -> usize {
        self.get_meta().total
    }

    fn get_meta(&self) -> &Meta;
    fn get_inner(&self) -> &Vec<Self::Inner>;
    fn get_inner_mut(&mut self) -> &mut Vec<Self::Inner>;
}

#[async_trait]
pub trait EsrFrom: Sized + Sync + Send + DeserializeOwned {
    // url=id by default
    fn url_from_id(id: &str) -> String {
        String::from(id)
    }

    fn url_from_id_and_token(id: &str, token: &str) -> String {
        let url = Self::url_from_id(id);
        if url.find('?').is_some() {
            url + "&access_token=" + token
        } else {
            url + "?access_token=" + token
        }
    }

    async fn bytes_from_url(url: &str) -> Result<Vec<u8>> {
        let client = get_static_client()?;
        log::debug!("Getting data from '{}'", url);

        // Creating an outgoing request.
        let mut response = client.get_async(url).await?;
        let mut buf = Vec::with_capacity(64*1024);
        response.copy_to(&mut buf).await?;

        log::debug!("Got data from '{}' (len={})", url, buf.len());

        Ok(buf)
    }

    async fn bytes_from_id(id: &str) -> Result<Vec<u8>> {
        Self::bytes_from_url(&*Self::url_from_id(id)).await
    }

    async fn bytes_from_id_with_token(id: &str, token: &str) -> Result<Vec<u8>> {
        Self::bytes_from_url(&*Self::url_from_id_and_token(id, token)).await
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Deserialize
        let info = serde_json::from_slice(bytes)?;
        Ok(info)
    }

    async fn from_url(url: &str) -> Result<Self> {
        let bytes = Self::bytes_from_url(url).await?;
        Self::from_bytes(&*bytes)
    }

    async fn from_id(id: &str) -> Result<Self> {
        Self::from_url(&*Self::url_from_id(id)).await
    }

    async fn from_id_with_token(id: &str, token: &str) -> Result<Self> {
        Self::from_url(&*Self::url_from_id_and_token(id, token)).await
    }

    // Owned arguments variants to allow use in task::spawn
    async fn from_url_owned(url: String) -> Result<Self> {
        Self::from_url(&*url).await
    }

    async fn from_id_owned(id: String) -> Result<Self> {
        Self::from_id(&*id).await
    }

    async fn from_id_with_token_owned(id: String, token: String) -> Result<Self> {
        Self::from_id_with_token(&*id, &*token).await
    }
}
