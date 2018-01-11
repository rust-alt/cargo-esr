/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use term_painter::{Style, ToStyle};
use term_painter::Attr::{Plain, Bold};
use term_painter::Color::{Red, Cyan, Yellow, Green, Blue};

use esr_errors::*;

#[derive(Clone)]
pub struct EsrFormatter {
    style: Style,
    text: String,
    trail: String,
}

impl EsrFormatter {
    pub fn new(style: Style, text: &str, trail: &str) -> Self {
        Self {
            style,
            text: String::from(text),
            trail: String::from(trail),
        }
    }

    pub fn trail_only(trail: &str) -> Self {
        Self {
            style: Plain.to_style(),
            text: String::new(),
            trail: String::from(trail),
        }
    }

    pub fn empty() -> Self {
        Self {
            style: Plain.to_style(),
            text: String::new(),
            trail: String::new(),
        }
    }

    pub fn print(&self, formatted: bool) {
        if formatted {
            print!("{}{}", self.style.paint(&self.text), self.trail);
        } else {
            print!("{}{}", &self.text, self.trail);
        }
    }

    pub fn print_grp(grp: &[Self], formatted: bool) {
        for f in grp {
            f.print(formatted);
        }
        println!("");
    }
}


pub struct EsrPrinter;

impl EsrPrinter {
    pub fn msg_pair(msg: &str, val: &str) -> [EsrFormatter; 2] {
        [
            EsrFormatter::new(Cyan.bold(), msg, ": "),
            EsrFormatter::new(Bold.to_style(), val, "\n "),
        ]
    }

    pub fn msg_pair_complex(msg: &str, val: &[EsrFormatter]) -> Vec<EsrFormatter> {
        let mut ret = Vec::with_capacity(val.len() + 1);
        ret.push(EsrFormatter::new(Cyan.bold(), msg, ": "));
        ret.extend_from_slice(val);
        ret
    }

    pub fn id(id: &str) -> EsrFormatter {
        EsrFormatter::new(Blue.bold(), id, " ")
    }

    pub fn err(val: &str) -> EsrFormatter {
        EsrFormatter::new(Red.bold(), val, "\n")
    }

    pub fn all_yanked() -> EsrFormatter {
        EsrFormatter::new(Red.bold(), "(empty/all yanked)", "\n ")
    }

    pub fn desc(orig_desc: &str) -> String {
        let desc = String::from(orig_desc);

        // Replace white-space with a single space
        let mut tmp = desc.replace(&['\t', '\n'][..], " ");
        let fixed_ws = loop {
            let curr = tmp.replace("  ", " ");
            if curr == tmp {
                break curr.trim().to_string();
            }
            tmp = curr;
        };

        // Multi-line
        let mut last_n = fixed_ws.rfind('\n').unwrap_or(0);
        let mut multi_line = fixed_ws;

        while multi_line.len() - last_n > 64 {
            let new_n = multi_line
                .get(last_n+1..last_n+64)
                .and_then(|slice| slice.rfind(' '))
                .unwrap_or(last_n);

            if new_n != last_n {
                assert!(last_n+new_n+1 < multi_line.len());
                unsafe { multi_line.as_mut_vec()[last_n+new_n+1] = b'\0' };
                multi_line = multi_line.replace('\0', "\n              ");
                last_n += new_n + " Discription: ".len();
            }
            else {
                break;
            }
        }

        multi_line
    }

    pub fn release(ver_opt: Option<&str>, age_res_opt: Option<Result<f64>>) -> String {
        match (ver_opt, age_res_opt) {
            (Some(ver), Some(Ok(age))) => format!("{} (released {:.1} months ago)", ver, age),
            _ => "N/A".into(),
        }
    }

    pub fn releases(stable: usize, non_yanked_pre: usize, yanked: usize) -> [EsrFormatter; 3] {
        [
            EsrFormatter::new(Green.bold(), &format!("{}", stable), "+"),
            EsrFormatter::new(Yellow.bold(), &format!("{}", non_yanked_pre), "+"),
            EsrFormatter::new(Red.bold(), &format!("{}", yanked), "\n "),
        ]
    }

    pub fn score_error(msg: &str) -> [EsrFormatter; 2] {
        [
            EsrFormatter::new(Red.bold(), msg, ": "),
            EsrFormatter::new(Bold.to_style(), "Error", "\n "),
        ]
    }

    pub fn score_na(msg: &str) -> [EsrFormatter; 2] {
        Self::msg_pair(msg, "N/A")
    }

    pub fn score_overview(msg: &str, pos: f64, neg: f64) -> [EsrFormatter; 4] {
        [
            EsrFormatter::new(Cyan.bold(), msg, ": "),
            EsrFormatter::new(Yellow.bold(), &format!("{:.3}", pos + neg) , " ("),
            EsrFormatter::new(Green.bold(), &format!("+{:.3}", pos) , " / "),
            EsrFormatter::new(Red.bold(), &format!("{:.3}", neg) , ")\n "),
        ]
    }

    pub fn score_details(msg: &str, table: &[(String, String, String)]) -> Vec<EsrFormatter> {
        let msg = format!("{: ^49}", msg);
        let frame = format!("{: ^49}", "-".repeat(msg.len()));

        let mut score_formatted = Vec::with_capacity(4096);
        score_formatted.push(EsrFormatter::new(Cyan.bold(), &*frame, "\n"));
        score_formatted.push(EsrFormatter::new(Cyan.bold(), &*msg, "\n"));
        score_formatted.push(EsrFormatter::new(Cyan.bold(), &*frame, "\n"));

        for line in table {
            if line.1.find("* -").is_some() {
                score_formatted.push(EsrFormatter::new(Yellow.bold(), &*line.0, " | "));
                score_formatted.push(EsrFormatter::new(Red.bold(), &*line.1, " | "));
                score_formatted.push(EsrFormatter::new(Red.bold(), &*line.2, "\n"));
            } else {
                score_formatted.push(EsrFormatter::new(Yellow.bold(), &*line.0, " | "));
                score_formatted.push(EsrFormatter::new(Green.bold(), &*line.1, " | "));
                score_formatted.push(EsrFormatter::new(Green.bold(), &*("+".to_string() + &*line.2), "\n"));
            }
        }
        score_formatted
    }

    pub fn crate_no_score(id: &str, e: &Error) {
        let msg = format!("Failed to get scores for crate \"{}\": {}.", id, e);
        println!("{}", Red.bold().paint(&msg));
    }

    pub fn repo_no_score(repo: &str, e: &Error) {
        let msg = format!("Failed to get scores for repo \"{}\": {}.", repo, e);
        println!("{}", Red.bold().paint(&msg));
    }

    pub fn search_no_results(search_pattern: &str) {
        let msg = format!("Searching for \"{}\" returned no results.", search_pattern);
        println!("{}", Yellow.bold().paint(&msg));
    }

    pub fn search_failed(search_pattern: &str, e: &Error) {
        let msg = format!("Search for \"{}\" failed.\n{}", search_pattern, backtrace_msg!(e));
        println!("{}", Red.bold().paint(&msg));
    }

    pub fn limit_out_of_range(limit: usize, min: usize, max: usize) {
        let msg = format!("{} is out of the range of valid limits. \
                          Please pass a value between {} and {}.", limit, min, max);
        println!("{}", Yellow.bold().paint(&msg));
    }

    pub fn limit_invalid(limit: &str) {
        let msg = format!("\"{}\" is an invalid limit value.", limit);
        println!("{}", Yellow.bold().paint(&msg));
    }

    pub fn no_token() {
        let msg = "Accessing GitHub's API wothout hitting rate-limits requires providing an access\
                   token.\n\n\
                   You can pass a token via -g/--gh-token option.\n\
                   Or by setting the variable CARGO_ESR_GH_TOKEN in the environment.\n\n\
                   To a acquire an access token, visit: <https://github.com/settings/tokens/new>\n\n\
                   Alternatively, you can pass -o/--crate-only to skip getting repository info.";
        println!("{}", Yellow.bold().paint(msg));
    }
}
