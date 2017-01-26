/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use ansi_term::Style;
use ansi_term::Colour::{Red, Cyan, Yellow, Green, Blue};


#[derive(Clone, Copy)]
pub struct EsrPrinter {
    is_tty: bool,
}

impl EsrPrinter {
    pub fn new_with_term(is_tty: bool) -> Self {
        Self { is_tty }
    }

    fn bold(&self, val: &str) -> String {
        if self.is_tty {
            format!("{}", Style::new().bold().paint(val))
        } else {
            val.into()
        }
    }

    fn yellow_bold(&self, val: &str) -> String {
        if self.is_tty {
            format!("{}", Yellow.bold().paint(val))
        } else {
            val.into()
        }
    }

    fn green_bold(&self, val: &str) -> String {
        if self.is_tty {
            format!("{}", Green.bold().paint(val))
        } else {
            val.into()
        }
    }

    fn red_bold(&self, val: &str) -> String {
        if self.is_tty {
            format!("{}", Red.bold().paint(val))
        } else {
            val.into()
        }
    }

    fn cyan_bold(&self, val: &str) -> String {
        if self.is_tty {
            format!("{}", Cyan.bold().paint(val))
        } else {
            val.into()
        }
    }

    pub fn blue_bold(&self, val: &str) -> String {
        if self.is_tty {
            format!("{}", Blue.bold().paint(val))
        } else {
            val.into()
        }
    }

    pub fn msg_pair(&self, msg: &str, val: &str) -> String {
        format!("{}: {}",
                self.cyan_bold(msg),
                self.bold(val))
    }

    pub fn score_error(&self, msg: &str) -> String {
        format!("{}: {}",
                self.red_bold(msg),
                self.bold("Error"))
    }

    pub fn score_na(&self, msg: &str) -> String {
        self.msg_pair(msg, "N/A")
    }

    pub fn score_overview(&self, msg: &str, pos: f64, neg: f64) -> String {
        format!("{}: {} ({} / {})",
                self.cyan_bold(msg),
                self.yellow_bold(&format!("{:.3}", pos + neg)),
                self.green_bold(&format!("+{:.3}", pos)),
                self.red_bold(&format!("{:.3}", neg)))
    }

    pub fn score_details(&self, msg: &str, table: &[(String, String, String)]) -> String {
        let msg = format!("{: ^49}", msg);
        let frame = format!("{: ^49}", "-".repeat(msg.len()));

        let mut score_str = String::with_capacity(4096);
        score_str.push_str(&format!("{}\n", self.cyan_bold(&*frame)));
        score_str.push_str(&format!("{}\n", self.cyan_bold(&*msg)));
        score_str.push_str(&format!("{}\n", self.cyan_bold(&*frame)));

        for line in table {
            if line.2.starts_with('-') {
                let line_str = format!("{} | {} | {}\n",
                                       self.yellow_bold(&*line.0),
                                       self.red_bold(&*line.1),
                                       self.red_bold(&*line.2));
                score_str.push_str(&line_str);

            } else {
                let line_str = format!("{} | {} | {}\n",
                                       self.yellow_bold(&*line.0),
                                       self.green_bold(&*line.1),
                                       self.green_bold(&*("+".to_string() + &*line.2)));
                score_str.push_str(&line_str);
            }
        }
        score_str
    }

    pub fn crate_no_score(&self, id: &str) {
        let msg = format!("Failed to get scores for crate \"{}\". Maybe it does not exist!", id);
        println!("{}", self.red_bold(&msg));
    }

    pub fn search_no_results(&self, search_pattern: &str) {
        let msg = format!("Searching for \"{}\" returned no results.", search_pattern);
        println!("{}", self.yellow_bold(&msg));
    }

    pub fn search_failed(&self, search_pattern: &str) {
        let msg = format!("Search for \"{}\" failed.", search_pattern);
        println!("{}", self.red_bold(&msg));
    }

    pub fn limit_out_of_range(&self, limit: u8, min: u8, max: u8) {
        let msg = format!("{} is out of the range of valid limits. \
                          Please pass a value between {} and {}.", limit, min, max);
        println!("{}", self.yellow_bold(&msg));
    }

    pub fn limit_invalid(&self, limit: &str) {
        let msg = format!("\"{}\" is an invalid limit value.", limit);
        println!("{}", self.yellow_bold(&msg));
    }

    pub fn no_token(&self) {
        let msg = "Accessing GitHub's API wothout hitting rate-limits requires providing an access\
                   token.\n\n\
                   You can pass a token via -g/--gh-token option.\n\
                   Or by setting the variable CARGO_ESR_GH_TOKEN in the environment.\n\n\
                   To a acquire an access token, visit: <https://github.com/settings/tokens/new>\n\n\
                   Alternatively, you can pass -o/--crate-only to skip getting repository info.";
        println!("{}", self.yellow_bold(msg));
    }
}
