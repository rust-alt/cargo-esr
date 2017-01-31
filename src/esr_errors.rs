/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use serde_json;
use hyper;
use native_tls;
use time;
use regex;

error_chain! {
    foreign_links {
        StdIO(::std::io::Error);
        Hyper(hyper::Error);
        NativeTls(native_tls::Error);
        TimeParse(time::ParseError);
        SerdeJson(serde_json::Error);
        Regex(regex::Error);
    }
}
