/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use pipeliner::Pipeline;

use esr_crate::{CrateInfoWithScore, CrateGeneralInfo};
use esr_github::RepoInfoWithScore;
use esr_printer::{EsrPrinter, EsrFormatter};
use esr_errors::*;

use std::f64;

pub struct CrateScores {
    crate_full: CrateInfoWithScore,
    repo_full: Option<Result<RepoInfoWithScore>>,
}

impl CrateScores {
    pub fn from_id(id: &str) -> Result<Self> {
        let crate_full = CrateInfoWithScore::from_id(id)?;
        let repo_full = crate_full.get_info().github_id()
            .map(|gh_id| RepoInfoWithScore::from_id(&gh_id));

        Ok(Self {
            crate_full,
            repo_full,
        })
    }

    pub fn from_id_with_token(id: &str, gh_token: &str) -> Result<Self> {
        let crate_full = CrateInfoWithScore::from_id(id)?;
        let repo_full =crate_full.get_info().github_id()
            .map(|gh_id| RepoInfoWithScore::from_id_with_token(&gh_id, gh_token));

        Ok(Self {
            crate_full,
            repo_full,
        })
    }

    pub fn from_id_crate_only(id: &str) -> Result<Self> {
        let crate_full = CrateInfoWithScore::from_id(id)?;

        Ok(Self {
            crate_full,
            repo_full: None,
        })
    }

    // ====================
    fn score_crate(&self) -> [EsrFormatter; 4] {
        let (pos, neg) = self.crate_full.get_score_tuple();
        EsrPrinter::score_overview("Crate Score", pos, neg)
    }

    fn score_repo(&self) -> Vec<EsrFormatter> {
        if let Some(ref repo_full_res) = self.repo_full {
            if let Ok(ref repo_full) = *repo_full_res {
                let (pos, neg) = repo_full.get_score_tuple();
                EsrPrinter::score_overview("Repo Score ", pos, neg).to_vec()
            } else {
                EsrPrinter::score_error("Repo Score ").to_vec()
            }
        } else {
            EsrPrinter::score_na("Repo Score ").to_vec()
        }
    }

    pub fn print_detailed_scores(&self, formatted: bool) {
        let table = self.crate_full.get_score_table();
        EsrFormatter::print_grp(&EsrPrinter::score_details("Crate Score Details", table), formatted);
        EsrFormatter::print_grp(&self.score_crate(), formatted);

        if let Some(ref repo_full_res) = self.repo_full {
            if let Ok(ref repo_full) = *repo_full_res {
                let table = repo_full.get_score_table();
                EsrFormatter::print_grp(&EsrPrinter::score_details("Repo Score Details", table), formatted);
            }
        }
        EsrFormatter::print_grp(&*self.score_repo(), formatted);
    }

    // =================

    pub fn collect_scores(crates: &[CrateGeneralInfo], token: &str, crate_only: bool, limit: usize) -> Vec<(String, Result<Self>)> {
        let id_token_pair: Vec<_> = crates.iter()
            .map(|cr| (String::from(cr.get_id()), String::from(token)))
            .collect();

        if crate_only {
            id_token_pair.into_iter()
                .with_threads(limit)
                .map(move |(id, _)| (id.clone(), CrateScores::from_id_crate_only(&id)))
                .collect()
        } else {
            id_token_pair.into_iter()
                .with_threads(limit)
                .map(move |(id, token)| (id.clone(), CrateScores::from_id_with_token(&id, &token)))
                .collect()
        }
    }

    fn info_pair(&self, id: &str, sort_positive: bool) -> (f64, Vec<EsrFormatter>) {
        let cr_info = self.crate_full.get_info();
        let (pos, neg) = self.crate_full.get_score_tuple();

        let empty_or_all_yanked_formatted = match cr_info.empty_or_all_yanked() {
            true => EsrPrinter::all_yanked(),
            false => EsrFormatter::trail_only("\n "),
        };

        let sort_score = match sort_positive {
            true => pos,
            false => pos + neg,
        };

        let releases = self.crate_full.get_score_info().get_releases();
        let non_yanked = self.crate_full.get_score_info().get_non_yanked_releases();
        let stable = self.crate_full.get_score_info().get_stable_releases();
        let yanked = releases - non_yanked;
        let non_yanked_pre = non_yanked - stable;
        let releases_formatted = EsrPrinter::releases(stable, non_yanked_pre, yanked);

        let max_ver = Some(cr_info.get_max_version());
        let max_ver_age = cr_info.max_version_age();
        let max_ver_msg = EsrPrinter::release(max_ver, max_ver_age);

        let last_stable_version = cr_info.last_stable_version();
        let last_stable_version_age = cr_info.last_stable_version_age();
        let last_stable_version_msg = EsrPrinter::release(last_stable_version, last_stable_version_age);

        let dependants = self.crate_full.get_score_info().get_dependants();
        let d_b_n_o = self.crate_full.get_score_info().get_dependants_from_non_owners();
        let dependants_msg = format!("{} ({} from non owners)", dependants, d_b_n_o);

        let mut info_formatter = Vec::with_capacity(32);
        info_formatter.push(EsrPrinter::id(id));
        info_formatter.push(empty_or_all_yanked_formatted);
        info_formatter.extend_from_slice(&self.score_crate());
        info_formatter.extend_from_slice(&self.score_repo());
        info_formatter.extend_from_slice(&*EsrPrinter::msg_pair_complex("Releases   ", &releases_formatted));
        info_formatter.extend_from_slice(&EsrPrinter::msg_pair("Max Version", &max_ver_msg));
        info_formatter.extend_from_slice(&EsrPrinter::msg_pair("Last Stable", &last_stable_version_msg));
        info_formatter.extend_from_slice(&EsrPrinter::msg_pair("Dependants ", &dependants_msg));
        info_formatter.extend_from_slice(&EsrPrinter::msg_pair("License    ", cr_info.get_license().unwrap_or("N/A")));
        info_formatter.extend_from_slice(&EsrPrinter::msg_pair("Repository ", cr_info.get_repository().unwrap_or("N/A")));
        info_formatter.extend_from_slice(&EsrPrinter::msg_pair("Description", cr_info.get_description().unwrap_or("N/A").trim()));

        (sort_score, info_formatter)
    }

    pub fn print_search_results(results: &[(String, Result<Self>)], sort_positive: bool, limit: usize, formatted: bool) {
        let mut results_vec = Vec::with_capacity(32);
        for res in results {
            match *res {
                (ref id, Ok(ref score_info)) => {
                    results_vec.push(score_info.info_pair(id, sort_positive));
                },
                (ref id, Err(_)) => {
                    results_vec.push((f64::MIN, vec![EsrPrinter::err(&format!("{}: Failed to get score info.", id))]));
                },
            }
        }

        // Negation to get scores in reverse.
        // `* 10000.0` to not lose order accuracy after casting.
        results_vec.sort_by_key(|&(sort_score, _)| -(sort_score * 10000.0) as i64);
        for (num, result) in results_vec.iter().take(limit).enumerate() {
            EsrPrinter::id(&format!("({})", num + 1)).print(formatted);
            EsrFormatter::print_grp(&result.1, formatted);
        }
    }
}
