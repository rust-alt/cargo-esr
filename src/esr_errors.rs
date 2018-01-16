/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use serde_json;
use time;
use regex;
use reqwest;

pub(crate) use failure::Fail;
pub(crate) type Result<T> = ::std::result::Result<T, EsrError>;

pub(crate) trait EsrFailExt : Fail + Sized {
    fn causes_msg(&self) -> String {
        let mut msg = String::with_capacity(4096);
        for c in self.causes() {
            msg.push_str("\n ");
            msg.push_str(&c.to_string());
        }

        if msg != "" {
            "Causes:".to_string() + &msg
        }
        else {
            msg
        }
    }
}

#[derive(Fail, Debug)]
pub enum EsrError {
    #[fail(display = "IO Error: {}.", _0)]
    StdIO(::std::io::Error),
    #[fail(display = "Time parsing Error: {}.", _0)]
    TimeParse(time::ParseError),
    #[fail(display = "Deserialization Error: {}.", _0)]
    SerdeJson(serde_json::Error),
    #[fail(display = "Regex Error: {}.", _0)]
    Regex(regex::Error),
    #[fail(display = "Reqwest Error: {}.", _0)]
    Reqwest(reqwest::Error),
    #[fail(display = "Error: {}.", _0)]
    Other(String),
}


impl EsrFailExt for EsrError {}

impl From<::std::io::Error> for EsrError {
    fn from(e: ::std::io::Error) -> Self {
        EsrError::StdIO(e)
    }
}

impl From<time::ParseError> for EsrError {
    fn from(e: time::ParseError) -> Self {
        EsrError::TimeParse(e)
    }
}

impl From<serde_json::Error> for EsrError {
    fn from(e: serde_json::Error) -> Self {
        EsrError::SerdeJson(e)
    }
}

impl From<regex::Error> for EsrError {
    fn from(e: regex::Error) -> Self {
        EsrError::Regex(e)
    }
}

impl From<reqwest::Error> for EsrError {
    fn from(e: reqwest::Error) -> Self {
        EsrError::Reqwest(e)
    }
}

impl<'a> From<&'a str> for EsrError {
    fn from(e: &str) -> Self {
        EsrError::Other(e.into())
    }
}
