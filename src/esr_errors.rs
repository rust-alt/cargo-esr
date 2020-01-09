/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use std::{fmt, result};

pub type Result<T> = result::Result<T, EsrError>;

#[derive(Debug)]
pub enum EsrError {
    StdIO(::std::io::Error),
    TimeParse(time::ParseError),
    SerdeJson(serde_json::Error),
    Regex(regex::Error),
    Reqwest(reqwest::Error),
    CratesIndex(String),
    TokioTaskJoin(tokio::task::JoinError),
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
            EsrError::CratesIndex(ref e) => write!(f, "CratesIndex Error: {}", e),
            EsrError::TokioTaskJoin(ref e) => write!(f, "tokio task join Error: {}", e),
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

impl From<tokio::task::JoinError> for EsrError {
    fn from(e: tokio::task::JoinError) -> Self {
        EsrError::TokioTaskJoin(e)
    }
}

impl From<reqwest::Error> for EsrError {
    fn from(e: reqwest::Error) -> Self {
        EsrError::Reqwest(e)
    }
}

impl From<crates_index::Error> for EsrError {
    fn from(e: crates_index::Error) -> Self {
        EsrError::CratesIndex(e.to_string())
    }
}

impl<'a> From<&'a str> for EsrError {
    fn from(e: &str) -> Self {
        EsrError::Other(e.into())
    }
}
