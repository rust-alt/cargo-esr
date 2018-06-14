/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use esr_errors::{Result, EsrError};
use tty_string::{TtyString, TtyStyle};
use tty_string::color as C;

const BOLD : fn() -> TtyStyle = || TtyStyle::bold();
const RED_BOLD : fn() -> TtyStyle = || TtyStyle::fg(C::RED).with_bold();
const GREEN_BOLD : fn() -> TtyStyle = || TtyStyle::fg(C::GREEN).with_bold();
const BLUE_BOLD : fn() -> TtyStyle = || TtyStyle::fg(C::BLUE).with_bold();
const YELLOW_BOLD : fn() -> TtyStyle = || TtyStyle::fg(C::YELLOW).with_bold();
const CYAN_BOLD : fn() -> TtyStyle = || TtyStyle::fg(C::CYAN).with_bold();

pub struct EsrPrinter;

impl EsrPrinter {
    pub fn msg_pair(msg: &str, val: impl Into<TtyString>) -> TtyString {
        let val_bold = val.into().with_style(TtyStyle::bold());
        TtyString::new(CYAN_BOLD(), msg) + ": " + val_bold  + "\n "
    }

    pub fn id(id: &str) -> TtyString {
        TtyString::new(BLUE_BOLD(), id)
    }

    pub fn err(val: &str) -> TtyString {
        TtyString::new(RED_BOLD(), val)
    }

    pub fn all_yanked() -> TtyString {
        TtyString::new(RED_BOLD(), "(empty/all yanked)")
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

    pub fn releases(stable: usize, non_yanked_pre: usize, yanked: usize) -> TtyString {
        let pos_sign = || TtyString::new(BOLD(), "+");
        let stable_f = TtyString::new(GREEN_BOLD(), &format!("{}", stable));
        let non_yanked_pre_f = TtyString::new(YELLOW_BOLD(), &format!("{}", non_yanked_pre));
        let yanked_f = TtyString::new(RED_BOLD(), &format!("{}", yanked));
        stable_f + pos_sign() + non_yanked_pre_f + pos_sign() + yanked_f
    }

    pub fn score_error(msg: &str) -> TtyString {
        TtyString::new(RED_BOLD(), msg) + ": " + TtyString::new(TtyStyle::bold(), "Error") + "\n "
    }

    pub fn score_na(msg: &str) -> TtyString {
        Self::msg_pair(msg, "N/A")
    }

    pub fn score_overview(msg: &str, pos: f64, neg: f64) -> TtyString {
        let b = |x| TtyString::new(BOLD(), x);
        let score_f = TtyString::new(YELLOW_BOLD(), &format!("{:.3}", pos + neg));
        let score_pos_f = TtyString::new(RED_BOLD(), &format!("{:.3}", neg));
        let score_neg_f = TtyString::new(GREEN_BOLD(), &format!("+{:.3}", pos));

        let tail = score_f + b(" (") + score_pos_f + b(" / ") + score_neg_f + b(")");
        Self::msg_pair(msg, tail)
    }

    pub fn score_details(msg: &str, table: &[(String, String, String)]) -> TtyString {
        let msg = format!("|{: ^83}|", msg);
        let frame ="-".repeat(85);

        let sep = || TtyString::new(CYAN_BOLD(), "| ");
        let frame_line = || TtyString::new(CYAN_BOLD(), &frame) + "\n";

        let mut score_formatted = "".into();
        score_formatted += frame_line();
        score_formatted += TtyString::new(CYAN_BOLD(), &*msg) + "\n";
        score_formatted += frame_line();

        for line in table {
            if line.1.find("* -").is_some() {
                score_formatted += sep() + TtyString::new(YELLOW_BOLD(), &*line.0) + sep();
                score_formatted += TtyString::new(RED_BOLD(), &*line.1) + sep();
                score_formatted += TtyString::new(RED_BOLD(), &format!("{: ^11}", line.2)) + sep() + "\n";
                score_formatted += frame_line();
            } else {
                score_formatted += sep() + TtyString::new(YELLOW_BOLD(), &*line.0) + sep();
                score_formatted += TtyString::new(GREEN_BOLD(), &*line.1) + sep();
                score_formatted += TtyString::new(GREEN_BOLD(), &format!("{: ^11}", "+".to_string() + &line.2)) + sep() + "\n";
                score_formatted += frame_line();
            }
        }

        score_formatted
    }

    pub fn crate_no_score(id: &str, e: &EsrError) -> TtyString {
        let msg = format!("{}.\nFailed to get scores for crate \"{}\". Maybe it does not exist.", e, id);
        TtyString::new(RED_BOLD(), &msg)
    }

    pub fn repo_no_score(repo: &str, e: &EsrError) -> TtyString {
        let msg = format!("{}.\nFailed to get scores for repo \"{}\". Maybe it does not exist.", e, repo);
        TtyString::new(RED_BOLD(), &msg)
    }

    pub fn search_no_results(search_pattern: &str) -> TtyString {
        let msg = format!("Searching for \"{}\" returned no results.", search_pattern);
        TtyString::new(YELLOW_BOLD(), &msg)
    }

    pub fn search_failed(search_pattern: &str, e: &EsrError) -> TtyString {
        let msg = format!("{}.\nSearch for \"{}\" failed.", e, search_pattern);
        TtyString::new(RED_BOLD(), &msg)
    }

    pub fn limit_out_of_range(limit: usize, min: usize, max: usize) -> TtyString {
        let msg = format!("{} is out of the range of valid limits. \
                          Please pass a value between {} and {}.", limit, min, max);
        TtyString::new(YELLOW_BOLD(), &msg)
    }

    pub fn limit_invalid(limit: &str) -> TtyString {
        let msg = format!("\"{}\" is an invalid limit value.", limit);
        TtyString::new(YELLOW_BOLD(), &msg)
    }

    pub fn no_token() -> TtyString {
        let msg = "Accessing GitHub's API wothout hitting rate-limits requires providing an access\
                   token.\n\n\
                   You can pass a token via -g/--gh-token option.\n\
                   Or by setting the variable CARGO_ESR_GH_TOKEN in the environment.\n\n\
                   To a acquire an access token, visit: <https://github.com/settings/tokens/new>\n\n\
                   Alternatively, you can pass -o/--crate-only to skip getting repository info.";
        TtyString::new(YELLOW_BOLD(), msg)
    }
}
