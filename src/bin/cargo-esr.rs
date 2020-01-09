/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use clap::{App, ArgGroup};
use clap::load_yaml;

use cargo_esr::esr_crate::CrateSearch;
use cargo_esr::esr_score::Scores;
use cargo_esr::esr_printer::EsrPrinter;

use std::env;

const LIMIT_LOW: usize = 5;
const LIMIT_HIGH: usize = 100;

fn check_limit(limit: &str) -> usize {
    match str::parse::<usize>(limit) {
        Ok(limit_num) => {
            let ll = LIMIT_LOW;
            let lh = LIMIT_HIGH;
            if limit_num < ll || limit_num > lh {
                EsrPrinter::limit_out_of_range(limit_num, ll, lh).println();
                std::process::exit(1);
            } else {
                limit_num
            }
        },
        Err(_) => {
            EsrPrinter::limit_invalid(limit).println();
            std::process::exit(1);
        },
    }
}



async fn run() {
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

    let search_by_relevance = m.is_present("search-by-relevance");
    let search_by_recent_downloads = m.is_present("search-by-recent-downloads");
    let search_by_total_downloads = m.is_present("search-by-total-downloads");

    if m.is_present("debug") {
        let _logger_setup = fern::Dispatch::new()
            .format(|out, message, _| {
                out.finish(format_args!(
                        "{} {}",
                        chrono::Local::now().format("[%H:%M:%S]"),
                        message
                ))
            })
            .level(log::LevelFilter::Off)
            //.level_for("reqwest", log::LevelFilter::Debug)
            .level_for("cargo_esr", log::LevelFilter::Debug)
            .chain(std::io::stderr())
            .apply();

        if _logger_setup.is_err() {
            EsrPrinter::err("Logger setup failed.");
            std::process::exit(1);
        }
    }

    let results_limit_num = check_limit(results_limit);
    let _search_limit_num = check_limit(search_limit);

    let mut gh_token = String::with_capacity(48);
    if m.value_of("gh-score").is_some() || !crate_only {
        if let Some(arg_token) = m.value_of("gh-token") {
            gh_token.push_str(arg_token);
        } else if let Ok(env_token) = std::env::var("CARGO_ESR_GH_TOKEN") {
            gh_token.push_str(&env_token);
        } else {
            EsrPrinter::no_token().println();
            std::process::exit(1);
        }
    }

    match (m.value_of("gh-score"), m.value_of("score"), m.values_of("search")) {
        (Some(repo_path), _, _)  => {
            match Scores::from_repo_with_token(repo_path.into(), gh_token).await {
                Ok(repo_scores) => repo_scores.detailed_scores().println(),
                Err(ref e) => {
                    EsrPrinter::repo_no_score(repo_path, e).println();
                    std::process::exit(1);
                },
            }
        },
        (_, Some(crate_name), _) => {
            let crates_scores_res = match (crate_only, repo_only) {
                (false, false) => Scores::from_id_with_token(crate_name.into(), gh_token).await,
                (true, false)  => Scores::from_id_crate_only(crate_name.into()).await,
                (false, true)  => Scores::from_id_with_token_repo_only(crate_name.into(), gh_token).await,
                (true, true)   => unreachable!(),
            };

            match crates_scores_res {
                Ok(crate_scores) => crate_scores.detailed_scores().println(),
                Err(ref e) => {
                    EsrPrinter::crate_no_score(crate_name, e).println();
                    std::process::exit(1);
                },
            }
        },

        (_, _, Some(search_pattern)) => {
            let search_str = search_pattern.fold(String::with_capacity(128), |s, p| s + p + "+")
                // In case a multi-word search is quoted
                .replace(' ', "+")
                .trim_end_matches('+')
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

            match CrateSearch::from_id_single_page(&search_args).await {
                Ok(search) => {
                    let crates = search.get_crates();

                    if crates.is_empty() {
                        EsrPrinter::search_no_results(&search_str).println();
                        std::process::exit(1);
                    }

                    let crates_scores_res = match Scores::collect_scores(
                        crates,
                        &gh_token,
                        crate_only,
                        repo_only)
                        .await {
                            Ok(res) => res,
                            Err(_) => {
                                EsrPrinter::err("A tokio task or more returned errors.");
                                std::process::exit(1);
                            },
                    };

                    Scores::search_results(&*crates_scores_res, sort_positive, results_limit_num).println();
                },
                Err(ref e) => {
                    EsrPrinter::search_failed(&search_str, e).println();
                    std::process::exit(1);
                }
            }
        },

        (_, _, _) => unreachable!(),
    }
}

fn main() {

    // build runtime
    let runtime_res = tokio::runtime::Builder::new()
        .enable_all()
        //.basic_scheduler()
        .threaded_scheduler()
        .core_threads(16)
        .build();

    match runtime_res {
        Ok(mut runtime) => runtime.block_on(run()),
        Err(_) => {
            EsrPrinter::err("Failed to create tokio runtime.").println();
            std::process::exit(1);
        },
    }
}
