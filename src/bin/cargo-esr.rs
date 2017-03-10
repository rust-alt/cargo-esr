/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

#[macro_use]
extern crate clap;
extern crate isatty;
extern crate cargo_esr;

use clap::{App, ArgGroup};

use cargo_esr::esr_crate::CrateSearch;
use cargo_esr::esr_score::CrateScores;
use cargo_esr::esr_printer::EsrPrinter;

use std::env;

const LIMIT_LOW: usize = 2;
const LIMIT_HIGH: usize = 50;

fn check_limit(limit: &str) -> usize {
    match str::parse::<usize>(limit) {
        Ok(limit_num) => {
            let ll = LIMIT_LOW;
            let lh = LIMIT_HIGH;
            if limit_num < ll || limit_num > lh {
                EsrPrinter::limit_out_of_range(limit_num, ll, lh);
                std::process::exit(1);
            } else {
                limit_num
            }
        },
        Err(_) => {
            EsrPrinter::limit_invalid(limit);
            std::process::exit(1);
        },
    }
}

fn main() {
    // clap
    let mut args: Vec<_> = env::args().collect();

    // cargo subcommand compat
    if args.len() >= 2 && args[1] == "esr" {
        args[0] = "cargo-esr".into();
        let _ = args.remove(1);
    }

    let yaml = load_yaml!("cargo-esr.yml");
    let search_or_score = ArgGroup::with_name("search-or-score")
        .args(&["search", "score"])
        .required(true);

    let clap_app = App::from_yaml(yaml).group(search_or_score);

    let m = clap_app.get_matches_from(args);

    let crate_only = m.is_present("crate-only");
    let sort_positive = m.is_present("sort-positive");
    let results_limit = m.value_of("results-limit").unwrap_or("10");
    let search_limit = m.value_of("search-limit").unwrap_or("30");
    let formatted = isatty::stdout_isatty() && !m.is_present("no-color");

    let results_limit_num = check_limit(results_limit);
    let search_limit_num = check_limit(search_limit);

    let mut gh_token = String::with_capacity(48);
    if !crate_only {
        if let Some(arg_token) = m.value_of("gh-token") {
            gh_token.push_str(arg_token);
        } else if let Ok(env_token) = std::env::var("CARGO_ESR_GH_TOKEN") {
            gh_token.push_str(&env_token);
        } else {
            EsrPrinter::no_token();
            std::process::exit(1);
        }
    }

    match (m.value_of("score"), m.values_of("search")) {
        (Some(crate_name), _) => {
            let crate_scores_res = if crate_only {
                CrateScores::from_id_crate_only(crate_name)
            } else {
                CrateScores::from_id_with_token(crate_name, &gh_token)
            };

            if let Ok(crate_scores) = crate_scores_res {
                crate_scores.print_detailed_scores(formatted);
            } else {
                EsrPrinter::crate_no_score(crate_name);
                std::process::exit(1);
            }
        },

        (_, Some(search_pattern)) => {
            let search_str = search_pattern.fold(String::with_capacity(128), |s, p| s + p + "+")
                // In case a multi-word search is quoted
                .replace(' ', "+")
                .trim_right_matches('+')
                .to_string();

            let search_res = CrateSearch::from_id_single_page(&("per_page=".to_string() + search_limit + "&q=" +
                                                                &search_str));

            if let Ok(search) = search_res {
                let crates = search.get_crates();

                if crates.is_empty() {
                    EsrPrinter::search_no_results(&search_str);
                    std::process::exit(1);
                }

                let crates_scores_res = CrateScores::collect_scores(
                    crates,
                    &gh_token,
                    crate_only,
                    search_limit_num);
                CrateScores::print_search_results(
                    &*crates_scores_res,
                    sort_positive,
                    results_limit_num,
                    formatted);
            } else {
                EsrPrinter::search_failed(&search_str);
                std::process::exit(1);
            }
        },

        (_, _) => unreachable!(),
    }
}
