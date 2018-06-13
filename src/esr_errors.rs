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

use std::{fmt, result};

pub type Result<T> = result::Result<T, EsrError>;

#[derive(Debug)]
pub enum EsrError {
    StdIO(::std::io::Error),
    TimeParse(time::ParseError),
    SerdeJson(serde_json::Error),
    Regex(regex::Error),
    Reqwest(reqwest::Error),
    Other(String),
}

impl fmt::Display for EsrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EsrError::StdIO(ref e) => write!(f, "IO Error: {}", e),
            EsrError::TimeParse(ref e) => write!(f, "Time parsing Error: {}", e),
            EsrError::SerdeJson(ref e) => write!(f, "Deserialization Error: {}", e),
            EsrError::Regex(ref e) => write!(f, "Regex Error: {}", e),
            EsrError::Reqwest(ref e) => write!(f, "Reqwest Error: {}", e),
            EsrError::Other(ref e) => write!(f, "Error: {}", e),
        }
    }
}

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
