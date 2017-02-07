/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use serde_json;
use serde::Deserialize;

use hyper_native_tls::NativeTlsClient;
use hyper::client::{Client, RedirectPolicy};
use hyper::net::HttpsConnector;
use hyper::header::UserAgent;

use pipeliner::Pipeline;

use std::io::Read;
use std::collections::HashMap;

use esr_errors::*;

pub fn mk_client() -> Result<Client> {
    // Create a client.
    let tls_client = NativeTlsClient::new()?;
    let https_connector = HttpsConnector::new(tls_client);
    let mut hyper_client = Client::with_connector(https_connector);

    // client opts
    hyper_client.set_redirect_policy(RedirectPolicy::FollowAll);

    // ret
    Ok(hyper_client)
}

// QUIZ: Explain `'static` in this context.
pub trait EsrFromMulti: EsrFrom + Send + 'static {
    type Inner: Clone;

    fn from_url_multi_with_init_client(url: &str, client: &Client, multi_page: bool) -> Result<Self> {
        let url = url.to_string();

        let mut initial_self = Self::from_url_with_client(&url, client)?;
        let total = initial_self.total_from_meta()?;

        // per_page=100 is the maximum number allowed.
        // If total > 100, GET all pages and append all results to initial_self inner vec
        if multi_page && total > 100 {
            let num_pages = (total as f64 / 100.0).ceil() as usize;

            // with_threads() is provided by the pipeliner::Pipeline trait
            let more_pages = (2...num_pages)
                .map(move |page| url.clone() + &format!("&page={}", page))
                .with_threads(8)
                .map(|page_url| Self::from_url(&page_url));

            for page_res in more_pages {
                let page = page_res?;
                initial_self.get_inner_mut().extend_from_slice(&*page.get_inner());
            }

        }

        if multi_page && initial_self.get_inner().len() != total {
            Err("Total no. of results does not match total reported")?;
        }

        Ok(initial_self)
    }

    fn from_url_multi(url: &str, multi_page: bool) -> Result<Self> {
        let client = mk_client()?;
        Self::from_url_multi_with_init_client(url, &client, multi_page)
    }

    fn total_from_meta(&self) -> Result<usize> {
        let num = self.get_meta().get("total").ok_or("total num of dependants missing")?;
        Ok(*num)
    }

    fn get_meta(&self) -> &HashMap<String, usize>;
    fn get_inner(&self) -> &Vec<Self::Inner>;
    fn get_inner_mut(&mut self) -> &mut Vec<Self::Inner>;
}

pub trait EsrFrom: Sized + Deserialize {
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

    fn bytes_from_url_with_client(url: &str, client: &Client) -> Result<Vec<u8>> {
        // Creating an outgoing request.
        let mut resp = client.get(url)
            .header(UserAgent("cargo-esr/0.1".into()))
            .send()?;

        // Read the Response.
        let mut buf = Vec::with_capacity(256 * 1024);
        resp.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn bytes_from_url(url: &str) -> Result<Vec<u8>> {
        let client = mk_client()?;
        Self::bytes_from_url_with_client(url, &client)
    }

    fn bytes_from_id(id: &str) -> Result<Vec<u8>> {
        Self::bytes_from_url(&Self::url_from_id(id))
    }

    fn bytes_from_id_with_token(id: &str, token: &str) -> Result<Vec<u8>> {
        Self::bytes_from_url(&Self::url_from_id_and_token(id, token))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Deserialize
        let info = serde_json::from_slice(bytes)?;
        Ok(info)
    }

    fn from_url_with_client(url: &str, client: &Client) -> Result<Self> {
        let bytes = Self::bytes_from_url_with_client(url, client)?;
        Self::from_bytes(&*bytes)
    }

    fn from_url(url: &str) -> Result<Self> {
        let client = mk_client()?;
        Self::from_url_with_client(url, &client)
    }

    fn from_id(id: &str) -> Result<Self> {
        Self::from_url(&Self::url_from_id(id))
    }

    fn from_id_with_token(id: &str, token: &str) -> Result<Self> {
        Self::from_url(&Self::url_from_id_and_token(id, token))
    }

    fn from_id_with_client(id: &str, client: &Client) -> Result<Self> {
        Self::from_url_with_client(&Self::url_from_id(id), client)
    }

    fn from_id_with_token_and_client(id: &str, token: &str, client: &Client) -> Result<Self> {
        Self::from_url_with_client(&Self::url_from_id_and_token(id, token), client)
    }
}

#[derive(Deserialize)]
pub struct DefEsrFrom;
impl EsrFrom for DefEsrFrom {}
