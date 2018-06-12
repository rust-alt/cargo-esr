/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use esr_errors::{Result, EsrError};
use esr_formatter::EsrFormatter;

use term::{Attr, color};

const BOLD: [Attr; 1] = [Attr::Bold];
const CYAN_BOLD: [Attr; 2] = [Attr::Bold, Attr::ForegroundColor(color::CYAN)];
const RED_BOLD: [Attr; 2] = [Attr::Bold, Attr::ForegroundColor(color::RED)];
const BLUE_BOLD: [Attr; 2] = [Attr::Bold, Attr::ForegroundColor(color::BLUE)];
const GREEN_BOLD: [Attr; 2] = [Attr::Bold, Attr::ForegroundColor(color::GREEN)];
const YELLOW_BOLD: [Attr; 2] = [Attr::Bold, Attr::ForegroundColor(color::YELLOW)];

pub struct EsrPrinter;

impl EsrPrinter {
    pub fn msg_pair(msg: &str, val: impl Into<EsrFormatter>) -> EsrFormatter {
        EsrFormatter::new(CYAN_BOLD.into(), msg) + ": " + val.into().with_merged_style(&BOLD.into()) + "\n "
    }

    pub fn id(id: &str) -> EsrFormatter {
        EsrFormatter::new(BLUE_BOLD.into(), id)
    }

    pub fn err(val: &str) -> EsrFormatter {
        EsrFormatter::new(RED_BOLD.into(), val)
    }

    pub fn all_yanked() -> EsrFormatter {
        EsrFormatter::new(RED_BOLD.into(), "(empty/all yanked)")
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

    pub fn releases(stable: usize, non_yanked_pre: usize, yanked: usize) -> EsrFormatter {
        let stable_f = EsrFormatter::new(GREEN_BOLD.into(), &format!("{}", stable));
        let non_yanked_pre_f = EsrFormatter::new(YELLOW_BOLD.into(), &format!("{}", non_yanked_pre));
        let yanked_f = EsrFormatter::new(RED_BOLD.into(), &format!("{}", yanked));
        stable_f + "+" + non_yanked_pre_f + "+" + yanked_f
    }

    pub fn score_error(msg: &str) -> EsrFormatter {
        EsrFormatter::new(RED_BOLD.into(), msg) + ": " + EsrFormatter::new(BOLD.into(), "Error") + "\n "
    }

    pub fn score_na(msg: &str) -> EsrFormatter {
        Self::msg_pair(msg, "N/A")
    }

    pub fn score_overview(msg: &str, pos: f64, neg: f64) -> EsrFormatter {
        let score_f = EsrFormatter::new(YELLOW_BOLD.into(), &format!("{:.3}", pos + neg));
        let score_pos_f = EsrFormatter::new(RED_BOLD.into(), &format!("{:.3}", neg));
        let score_neg_f = EsrFormatter::new(GREEN_BOLD.into(), &format!("+{:.3}", pos));

        let tail = score_f + " (" + score_pos_f + " / " + score_neg_f + ")";
        Self::msg_pair(msg, tail)
    }

    pub fn score_details(msg: &str, table: &[(String, String, String)]) -> EsrFormatter {
        let msg = format!("|{: ^83}|", msg);
        let frame ="-".repeat(85);

        let sep = || EsrFormatter::new(CYAN_BOLD.into(), "| ");
        let frame_line = || EsrFormatter::new(CYAN_BOLD.into(), &frame) + "\n";

        let mut score_formatted = "".into();
        score_formatted += frame_line();
        score_formatted += EsrFormatter::new(CYAN_BOLD.into(), &*msg) + "\n";
        score_formatted += frame_line();

        for line in table {
            if line.1.find("* -").is_some() {
                score_formatted += sep() + EsrFormatter::new(YELLOW_BOLD.into(), &*line.0) + sep();
                score_formatted += EsrFormatter::new(RED_BOLD.into(), &*line.1) + sep();
                score_formatted += EsrFormatter::new(RED_BOLD.into(), &format!("{: ^11}", line.2)) + sep() + "\n";
                score_formatted += frame_line();
            } else {
                score_formatted += sep() + EsrFormatter::new(YELLOW_BOLD.into(), &*line.0) + sep();
                score_formatted += EsrFormatter::new(GREEN_BOLD.into(), &*line.1) + sep();
                score_formatted += EsrFormatter::new(GREEN_BOLD.into(), &format!("{: ^11}", "+".to_string() + &line.2)) + sep() + "\n";
                score_formatted += frame_line();
            }
        }

        score_formatted
    }

    pub fn crate_no_score(id: &str, e: &EsrError, formatted: bool) {
        let msg = format!("{}.\nFailed to get scores for crate \"{}\". Maybe it does not exist.", e, id);
        EsrFormatter::new(RED_BOLD.into(), &msg).println(formatted);
    }

    pub fn repo_no_score(repo: &str, e: &EsrError, formatted: bool) {
        let msg = format!("{}.\nFailed to get scores for repo \"{}\". Maybe it does not exist.", e, repo);
        EsrFormatter::new(RED_BOLD.into(), &msg).println(formatted);
    }

    pub fn search_no_results(search_pattern: &str, formatted: bool) {
        let msg = format!("Searching for \"{}\" returned no results.", search_pattern);
        EsrFormatter::new(YELLOW_BOLD.into(), &msg).println(formatted);
    }

    pub fn search_failed(search_pattern: &str, e: &EsrError, formatted: bool) {
        let msg = format!("{}.\nSearch for \"{}\" failed.", e, search_pattern);
        EsrFormatter::new(RED_BOLD.into(), &msg).println(formatted);
    }

    pub fn limit_out_of_range(limit: usize, min: usize, max: usize, formatted: bool) {
        let msg = format!("{} is out of the range of valid limits. \
                          Please pass a value between {} and {}.", limit, min, max);
        EsrFormatter::new(YELLOW_BOLD.into(), &msg).println(formatted);
    }

    pub fn limit_invalid(limit: &str, formatted: bool) {
        let msg = format!("\"{}\" is an invalid limit value.", limit);
        EsrFormatter::new(YELLOW_BOLD.into(), &msg).println(formatted);
    }

    pub fn no_token(formatted: bool) {
        let msg = "Accessing GitHub's API wothout hitting rate-limits requires providing an access\
                   token.\n\n\
                   You can pass a token via -g/--gh-token option.\n\
                   Or by setting the variable CARGO_ESR_GH_TOKEN in the environment.\n\n\
                   To a acquire an access token, visit: <https://github.com/settings/tokens/new>\n\n\
                   Alternatively, you can pass -o/--crate-only to skip getting repository info.";
        EsrFormatter::new(YELLOW_BOLD.into(), msg).println(formatted);
    }
}
