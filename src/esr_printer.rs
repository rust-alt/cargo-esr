/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use term::{self, Attr, color};

const BOLD: Option<&'static[term::Attr]> = Some(&[Attr::Bold]);
const CYAN_BOLD: Option<&'static[term::Attr]> = Some(&[Attr::Bold, Attr::ForegroundColor(color::CYAN)]);
const RED_BOLD: Option<&'static[term::Attr]> = Some(&[Attr::Bold, Attr::ForegroundColor(color::RED)]);
const BLUE_BOLD: Option<&'static[term::Attr]> = Some(&[Attr::Bold, Attr::ForegroundColor(color::BLUE)]);
const GREEN_BOLD: Option<&'static[term::Attr]> = Some(&[Attr::Bold, Attr::ForegroundColor(color::GREEN)]);
const YELLOW_BOLD: Option<&'static[term::Attr]> = Some(&[Attr::Bold, Attr::ForegroundColor(color::YELLOW)]);

//use esr_errors::{Result, EsrError, EsrFailExt};
use esr_errors::{Result, EsrError};

#[derive(Clone)]
pub struct EsrFormatter {
    style: Option<&'static[term::Attr]>,
    text: String,
    trail: String,
}

impl EsrFormatter {
    pub fn new(style: Option<&'static[term::Attr]>, text: &str, trail: &str) -> Self {
        Self {
            style,
            text: String::from(text),
            trail: String::from(trail),
        }
    }

    pub fn new_and_print(style: Option<&'static[term::Attr]>, text: &str, trail: &str, formatted: bool) {
        let new = Self {
            style,
            text: String::from(text),
            trail: String::from(trail),
        };
        new.print(formatted);
    }

    pub fn trail_only(trail: &str) -> Self {
        Self {
            style: None,
            text: String::new(),
            trail: String::from(trail),
        }
    }

    pub fn empty() -> Self {
        Self {
            style: None,
            text: String::new(),
            trail: String::new(),
        }
    }

    fn _print(&self, formatted: bool) -> Result<()> {
        match (formatted, term::stdout(), self.style) {
            (true, Some(mut out), Some(style)) => {
                for &s in style {
                    out.attr(s)?;
                }
                write!(out, "{}", self.text)?;
                out.reset()?;
                write!(out, "{}", self.trail)?;
                Ok(())
            },
            _ => {
                print!("{}{}", &self.text, self.trail);
                Ok(())
            },
        }
    }

    pub fn print(&self, formatted: bool) {
        self._print(formatted).expect("Printer failed. Something is wrong.");
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
            EsrFormatter::new(CYAN_BOLD, msg, ": "),
            EsrFormatter::new(BOLD, val, "\n "),
        ]
    }

    pub fn msg_pair_complex(msg: &str, val: &[EsrFormatter]) -> Vec<EsrFormatter> {
        let mut ret = Vec::with_capacity(val.len() + 1);
        ret.push(EsrFormatter::new(CYAN_BOLD, msg, ": "));
        ret.extend_from_slice(val);
        ret
    }

    pub fn id(id: &str) -> EsrFormatter {
        EsrFormatter::new(BLUE_BOLD, id, " ")
    }

    pub fn err(val: &str) -> EsrFormatter {
        EsrFormatter::new(RED_BOLD, val, "\n")
    }

    pub fn all_yanked() -> EsrFormatter {
        EsrFormatter::new(RED_BOLD, "(empty/all yanked)", "\n ")
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
            EsrFormatter::new(GREEN_BOLD, &format!("{}", stable), "+"),
            EsrFormatter::new(YELLOW_BOLD, &format!("{}", non_yanked_pre), "+"),
            EsrFormatter::new(RED_BOLD, &format!("{}", yanked), "\n "),
        ]
    }

    pub fn score_error(msg: &str) -> [EsrFormatter; 2] {
        [
            EsrFormatter::new(RED_BOLD, msg, ": "),
            EsrFormatter::new(BOLD, "Error", "\n "),
        ]
    }

    pub fn score_na(msg: &str) -> [EsrFormatter; 2] {
        Self::msg_pair(msg, "N/A")
    }

    pub fn score_overview(msg: &str, pos: f64, neg: f64) -> [EsrFormatter; 4] {
        [
            EsrFormatter::new(CYAN_BOLD, msg, ": "),
            EsrFormatter::new(YELLOW_BOLD, &format!("{:.3}", pos + neg) , " ("),
            EsrFormatter::new(GREEN_BOLD, &format!("+{:.3}", pos) , " / "),
            EsrFormatter::new(RED_BOLD, &format!("{:.3}", neg) , ")\n "),
        ]
    }

    pub fn score_details(msg: &str, table: &[(String, String, String)]) -> Vec<EsrFormatter> {
        let msg = format!("{: ^49}", msg);
        let frame = format!("{: ^49}", "-".repeat(msg.len()));

        let mut score_formatted = Vec::with_capacity(4096);
        score_formatted.push(EsrFormatter::new(CYAN_BOLD, &*frame, "\n"));
        score_formatted.push(EsrFormatter::new(CYAN_BOLD, &*msg, "\n"));
        score_formatted.push(EsrFormatter::new(CYAN_BOLD, &*frame, "\n"));

        for line in table {
            if line.1.find("* -").is_some() {
                score_formatted.push(EsrFormatter::new(YELLOW_BOLD, &*line.0, " | "));
                score_formatted.push(EsrFormatter::new(RED_BOLD, &*line.1, " | "));
                score_formatted.push(EsrFormatter::new(RED_BOLD, &*line.2, "\n"));
            } else {
                score_formatted.push(EsrFormatter::new(YELLOW_BOLD, &*line.0, " | "));
                score_formatted.push(EsrFormatter::new(GREEN_BOLD, &*line.1, " | "));
                score_formatted.push(EsrFormatter::new(GREEN_BOLD, &*("+".to_string() + &*line.2), "\n"));
            }
        }
        score_formatted
    }

    pub fn crate_no_score(id: &str, e: &EsrError, formatted: bool) {
        let msg = format!("{}.\nFailed to get scores for crate \"{}\". Maybe it does not exist.", e, id);
        EsrFormatter::new_and_print(RED_BOLD, &msg, "\n", formatted);
    }

    pub fn repo_no_score(repo: &str, e: &EsrError, formatted: bool) {
        let msg = format!("{}.\nFailed to get scores for repo \"{}\". Maybe it does not exist.", e, repo);
        EsrFormatter::new_and_print(RED_BOLD, &msg, "\n", formatted);
    }

    pub fn search_no_results(search_pattern: &str, formatted: bool) {
        let msg = format!("Searching for \"{}\" returned no results.", search_pattern);
        EsrFormatter::new_and_print(YELLOW_BOLD, &msg, "\n", formatted);
    }

    pub fn search_failed(search_pattern: &str, e: &EsrError, formatted: bool) {
        let msg = format!("{}.\nSearch for \"{}\" failed.", e, search_pattern);
        EsrFormatter::new_and_print(RED_BOLD, &msg, "\n", formatted);
    }

    pub fn limit_out_of_range(limit: usize, min: usize, max: usize, formatted: bool) {
        let msg = format!("{} is out of the range of valid limits. \
                          Please pass a value between {} and {}.", limit, min, max);
        EsrFormatter::new_and_print(YELLOW_BOLD, &msg, "\n", formatted);
    }

    pub fn limit_invalid(limit: &str, formatted: bool) {
        let msg = format!("\"{}\" is an invalid limit value.", limit);
        EsrFormatter::new_and_print(YELLOW_BOLD, &msg, "\n", formatted);
    }

    pub fn no_token(formatted: bool) {
        let msg = "Accessing GitHub's API wothout hitting rate-limits requires providing an access\
                   token.\n\n\
                   You can pass a token via -g/--gh-token option.\n\
                   Or by setting the variable CARGO_ESR_GH_TOKEN in the environment.\n\n\
                   To a acquire an access token, visit: <https://github.com/settings/tokens/new>\n\n\
                   Alternatively, you can pass -o/--crate-only to skip getting repository info.";
        EsrFormatter::new_and_print(YELLOW_BOLD, msg, "\n", formatted);
    }
}
