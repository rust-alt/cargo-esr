/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

#![feature(inclusive_range_syntax)]
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate failure_derive;
extern crate failure;

extern crate term_painter;
extern crate serde;
extern crate serde_json;
extern crate reqwest;
extern crate time;
extern crate semver;
extern crate pipeliner;
extern crate regex;

#[macro_use]
mod esr_macros;
mod esr_errors;
mod esr_from;
pub mod esr_util;
pub mod esr_crate;
pub mod esr_github;
pub mod esr_score;
pub mod esr_printer;
