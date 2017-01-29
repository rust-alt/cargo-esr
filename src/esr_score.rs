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
use esr_printer::EsrPrinter;
use esr_errors::*;

use std::f64;

pub struct CrateScores {
    crate_full: CrateInfoWithScore,
    repo_full: Option<Result<RepoInfoWithScore>>,
    printer: EsrPrinter,
}

impl CrateScores {
    pub fn from_id(id: &str, printer: EsrPrinter) -> Result<Self> {
        let crate_full = CrateInfoWithScore::from_id(id)?;
        let repo_full = if let Some(gh_id) = crate_full.get_info().github_id() {
            Some(RepoInfoWithScore::from_id(gh_id))
        } else {
            None
        };

        Ok(Self {
            crate_full,
            repo_full,
            printer,
        })
    }

    pub fn from_id_with_token(id: &str, gh_token: &str, printer: EsrPrinter) -> Result<Self> {
        let crate_full = CrateInfoWithScore::from_id(id)?;
        let repo_full = if let Some(gh_id) = crate_full.get_info().github_id() {
            Some(RepoInfoWithScore::from_id_with_token(gh_id, gh_token))
        } else {
            None
        };

        Ok(Self {
            crate_full,
            repo_full,
            printer,
        })
    }

    pub fn from_id_crate_only(id: &str, printer: EsrPrinter) -> Result<Self> {
        let crate_full = CrateInfoWithScore::from_id(id)?;

        Ok(Self {
            crate_full,
            repo_full: None,
            printer,
        })
    }

    // ====================
    fn score_crate(&self) -> String {
        let (pos, neg) = self.crate_full.get_score_tuple();
        self.printer.score_overview("Crate Score", pos, neg)
    }

    fn score_repo(&self) -> String {
        if let Some(ref repo_full_res) = self.repo_full {
            if let Ok(ref repo_full) = *repo_full_res {
                let (pos, neg) = repo_full.get_score_tuple();
                self.printer.score_overview("Repo Score ", pos, neg)
            } else {
                self.printer.score_error("Repo Score ")
            }
        } else {
            self.printer.score_na("Repo Score ")
        }
    }

    pub fn print_detailed_scores(&self) {
        let table = self.crate_full.get_score_table();
        println!("{}", self.printer.score_details("Crate Score Details", table));
        println!("{}", self.score_crate());

        if let Some(ref repo_full_res) = self.repo_full {
            if let Ok(ref repo_full) = *repo_full_res {
                let table = repo_full.get_score_table();
                println!("{}", self.printer.score_details("Repo Score Details", table));
            }
        }
        println!("{}", self.score_repo());
    }

    // =================

    pub fn collect_scores(crates: &[CrateGeneralInfo], token: &str, crate_only: bool, printer: EsrPrinter) -> Vec<(String, Result<Self>)> {
        let id_token_pair: Vec<_> = crates.iter()
            .map(|cr| (String::from(cr.get_id()), String::from(token)))
            .collect();

        if crate_only {
            id_token_pair.into_iter()
                .with_threads(10)
                .map(move |(id, _)| (id.clone(), CrateScores::from_id_crate_only(&id, printer)))
                .collect()
        } else {
            id_token_pair.into_iter()
                .with_threads(10)
                .map(move |(id, token)| (id.clone(), CrateScores::from_id_with_token(&id, &token, printer)))
                .collect()
        }
    }

    fn info_pair(&self, id: &str, sort_positive: bool) -> (f64, String) {
        let (pos, neg) = self.crate_full.get_score_tuple();

        let all_yanked_str = match self.crate_full.get_info().all_yanked() {
            true => self.printer.red_bold("(yanked)"),
            false => String::new(),
        };

        let sort_score = match sort_positive {
            true => pos,
            false => pos + neg,
        };

        let releases = self.crate_full.get_score_info().get_releases();
        let non_yanked = self.crate_full.get_score_info().get_non_yanked_releases();
        let yanked = releases - non_yanked;
        let m_s_l_r = self.crate_full.get_score_info().get_months_since_last_release();
        let releases_msg = format!("{}+{} ({:.1} months since last non-yanked release)",
                                   self.printer.green_bold(&format!("{}", non_yanked)),
                                   self.printer.red_bold(&format!("{}", yanked)),
                                   m_s_l_r);

        let dependants = self.crate_full.get_score_info().get_dependants();
        let d_b_n_o = self.crate_full.get_score_info().get_dependants_from_non_owners();
        let dependants_msg = format!("{} ({} from non owners)", dependants, d_b_n_o);

        let info_str = format!("{} {}\n  {}\n  {}\n  {}\n  {}\n  {}\n  {}\n  {}\n  {}\n",
                               self.printer.blue_bold(id),
                               all_yanked_str,
                               self.score_crate(),
                               self.score_repo(),
                               self.printer.msg_pair("Releases   ", &releases_msg),
                               self.printer.msg_pair("Dependants ", &dependants_msg),
                               self.printer.msg_pair("Max Version",
                                              self.crate_full.get_info().get_max_version()),
                               self.printer.msg_pair("License    ",
                                              self.crate_full.get_info().get_license().unwrap_or("N/A")),
                               self.printer.msg_pair("Repository ",
                                              self.crate_full.get_info().get_repository().unwrap_or("N/A")),
                               self.printer.msg_pair("Description",
                                              self.crate_full.get_info().get_description().unwrap_or("N/A")
                                              .trim()),
                               );

        (sort_score, info_str)
    }

    pub fn print_search_results(results: &[(String, Result<Self>)], sort_positive: bool, printer: EsrPrinter) {
        let mut results_vec = Vec::with_capacity(16);
        for res in results {
            match *res {
                (ref id, Ok(ref score_info)) => {
                    results_vec.push(score_info.info_pair(id, sort_positive));
                },
                (ref id, Err(_)) => {
                    results_vec.push((f64::MIN, format!("{}: Failed to get score info.", id)));
                },
            }
        }

        // -sort_score to get scores in reverse
        results_vec.sort_by_key(|&(sort_score, _)| -sort_score as i64);
        for (num, result) in results_vec.iter().enumerate() {
            println!("{} {}",
                     printer.blue_bold(&format!("({})", num + 1)),
                     result.1);
        }
    }
}
