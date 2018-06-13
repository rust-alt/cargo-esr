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
extern crate tty_string;
extern crate cargo_esr;

use clap::{App, ArgGroup};
use tty_string::TtyString;

use cargo_esr::esr_crate::CrateSearch;
use cargo_esr::esr_score::Scores;
use cargo_esr::esr_printer::EsrPrinter;

use std::env;

const LIMIT_LOW: usize = 5;
const LIMIT_HIGH: usize = 100;

fn check_limit(limit: &str, print: fn(&TtyString)) -> usize {
    match str::parse::<usize>(limit) {
        Ok(limit_num) => {
            let ll = LIMIT_LOW;
            let lh = LIMIT_HIGH;
            if limit_num < ll || limit_num > lh {
                print(&EsrPrinter::limit_out_of_range(limit_num, ll, lh));
                std::process::exit(1);
            } else {
                limit_num
            }
        },
        Err(_) => {
            print(&EsrPrinter::limit_invalid(limit));
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
        .args(&["search", "score", "gh-score"])
        .required(true);

    let search_by = ArgGroup::with_name("search-by")
        .args(&["search-by-relevance", "search-by-recent-downloads", "search-by-total-downloads"])
        .required(false);

    let score_filter = ArgGroup::with_name("score-filter")
        .args(&["crate-only", "repo-only"])
        .required(false);

    let clap_app = App::from_yaml(yaml)
        .group(search_or_score)
        .group(search_by)
        .group(score_filter);

    let m = clap_app.get_matches_from(args);

    let crate_only = m.is_present("crate-only");
    let repo_only = m.is_present("repo-only");
    let sort_positive = m.is_present("sort-positive");
    let results_limit = m.value_of("results-limit").unwrap_or("10");
    let search_limit = m.value_of("search-limit").unwrap_or("25");
    let formatted = isatty::stdout_isatty() && !m.is_present("no-color");

    let search_by_relevance = m.is_present("search-by-relevance");
    let search_by_recent_downloads = m.is_present("search-by-recent-downloads");
    let search_by_total_downloads = m.is_present("search-by-total-downloads");

    // Pick print method
    let print = match formatted {
        false => TtyString::println_plain,
        true  => TtyString::println,
    };

    let results_limit_num = check_limit(results_limit, print);
    let search_limit_num = check_limit(search_limit, print);

    let mut gh_token = String::with_capacity(48);
    if m.value_of("gh-score").is_some() || !crate_only {
        if let Some(arg_token) = m.value_of("gh-token") {
            gh_token.push_str(arg_token);
        } else if let Ok(env_token) = std::env::var("CARGO_ESR_GH_TOKEN") {
            gh_token.push_str(&env_token);
        } else {
            print(&EsrPrinter::no_token());
            std::process::exit(1);
        }
    }

    match (m.value_of("gh-score"), m.value_of("score"), m.values_of("search")) {
        (Some(repo_path), _, _)  => {
            match Scores::from_repo_with_token(repo_path, &gh_token) {
                Ok(repo_scores) => print(&repo_scores.detailed_scores()),
                Err(ref e) => {
                    print(&EsrPrinter::repo_no_score(repo_path, e));
                    std::process::exit(1);
                },
            }
        },
        (_, Some(crate_name), _) => {
            let crates_scores_res = match (crate_only, repo_only) {
                (false, false) => Scores::from_id_with_token(crate_name, &gh_token),
                (true, false)  => Scores::from_id_crate_only(crate_name),
                (false, true)  => Scores::from_id_with_token_repo_only(crate_name, &gh_token),
                (true, true)   => unreachable!(),
            };

            match crates_scores_res {
                Ok(crate_scores) => print(&crate_scores.detailed_scores()),
                Err(ref e) => {
                    print(&EsrPrinter::crate_no_score(crate_name, e));
                    std::process::exit(1);
                },
            }
        },

        (_, _, Some(search_pattern)) => {
            let search_str = search_pattern.fold(String::with_capacity(128), |s, p| s + p + "+")
                // In case a multi-word search is quoted
                .replace(' ', "+")
                .trim_right_matches('+')
                .to_string();

            let search_args = match search_str.is_empty() {
                true => match (search_by_relevance, search_by_total_downloads, search_by_recent_downloads) {
                    // default
                    (false, false, _) => "per_page=".to_string() + search_limit + "&sort=recent-downloads",
                    (_, true, _)      => "per_page=".to_string() + search_limit + "&sort=downloads",
                    (true, _, _)      => "per_page=".to_string() + search_limit,
                },
                false => match (search_by_relevance, search_by_total_downloads, search_by_recent_downloads) {
                    // default
                    (false, false, _) => "per_page=".to_string() + search_limit + "&q=" + &search_str + "&sort=recent-downloads",
                    (_, true, _)      => "per_page=".to_string() + search_limit + "&q=" + &search_str + "&sort=downloads",
                    (true, _, _)      => "per_page=".to_string() + search_limit + "&q=" + &search_str,
                },

            };

            match CrateSearch::from_id_single_page(&search_args) {
                Ok(search) => {
                    let crates = search.get_crates();

                    if crates.is_empty() {
                        print(&EsrPrinter::search_no_results(&search_str));
                        std::process::exit(1);
                    }

                    let crates_scores_res = Scores::collect_scores(
                        crates,
                        &gh_token,
                        crate_only,
                        repo_only,
                        search_limit_num);

                    print(&Scores::search_results(&*crates_scores_res, sort_positive, results_limit_num));
                },
                Err(ref e) => {
                    print(&EsrPrinter::search_failed(&search_str, e));
                    std::process::exit(1);
                }
            }
        },

        (_, _, _) => unreachable!(),
    }
}
