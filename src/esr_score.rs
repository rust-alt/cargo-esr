/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use crate::esr_crate::{CrateInfoWithScore, CrateInfo, CrateGeneralInfo};
use crate::esr_from::EsrFrom;
use crate::esr_github::RepoInfoWithScore;
use crate::esr_printer::EsrPrinter;
use crate::esr_util;
use crate::esr_errors::Result;

use term_string::TermString;
use tokio::task;

use std::f64;
use std::default::Default;

pub enum Scores {
    CrateAndRepo(CrateInfoWithScore, Result<RepoInfoWithScore>),
    CrateOnly(CrateInfoWithScore),
    RepoOnly(RepoInfoWithScore),
}

impl Scores {
    pub async fn from_id_with_token(id: String, gh_token: String) -> Result<Self> {
        let cr_info = CrateInfo::from_id(&*id).await?;

        let repo_score_res = cr_info.github_id()
            .ok_or("Failed to get GitHub id")
            .map(|gh_id| task::spawn(RepoInfoWithScore::from_id_with_token(gh_id, gh_token)));

        let cr_score = CrateInfoWithScore::from_info(cr_info).await?;

        match repo_score_res {
            Ok(repo_score) => Ok(Scores::CrateAndRepo(cr_score, repo_score.await?)),
            Err(_) => Ok(Scores::CrateOnly(cr_score)),
        }
    }

    pub async fn from_id_crate_only(id: String) -> Result<Self> {
        let cr_score = CrateInfoWithScore::from_id(id).await?;
        Ok(Scores::CrateOnly(cr_score))
    }

    pub async fn from_id_with_token_repo_only(id: String, gh_token: String) -> Result<Self> {
        let cr_score = CrateInfoWithScore::from_id(id).await?;
        let gh_id = cr_score.get_info().github_id().ok_or("repo-only score requested but failed to get GitHub id")?;
        let repo_score = RepoInfoWithScore::from_id_with_token(gh_id, gh_token).await?;

        Ok(Scores::RepoOnly(repo_score))
    }

    pub async fn from_repo_with_token(repo: String, gh_token: String) -> Result<Self> {
        let gh_id = esr_util::github_repo(&*repo).ok_or("repo score requested but failed to get a valid GitHub repo path")?;
        let repo_score = RepoInfoWithScore::from_id_with_token(gh_id, gh_token).await?;

        Ok(Scores::RepoOnly(repo_score))
    }

    // ====================
    fn score_crate(&self) -> TermString {
        match *self {
            Scores::CrateAndRepo(ref cr_score, _) | Scores::CrateOnly(ref cr_score) => {
                let (pos, neg) = cr_score.get_score_tuple();
                EsrPrinter::score_overview("Crate Score", pos, neg)
            },
            Scores::RepoOnly(_) => unreachable!(),
        }
    }

    fn score_repo(&self) -> TermString {
        match *self {
            Scores::CrateAndRepo (_, Ok(ref repo_score)) | Scores::RepoOnly(ref repo_score) => {
                let (pos, neg) = repo_score.get_score_tuple();
                EsrPrinter::score_overview("Repo Score ", pos, neg)
            },
            Scores::CrateAndRepo (_, Err(_)) => EsrPrinter::score_error("Repo Score "),
            Scores::CrateOnly(_) => EsrPrinter::score_na("Repo Score "),
        }
    }

    pub fn detailed_scores(&self) -> TermString {
        // Unfortunately, `if let` is not as powerful as `match`. So, we have to
        // to do this *_opt dance.
        let cr_score_opt = match *self {
            Scores::CrateAndRepo(ref cr_score, _) | Scores::CrateOnly(ref cr_score) => Some(cr_score),
            Scores::RepoOnly(_) => None,
        };

        let repo_score_opt = match *self {
            Scores::CrateAndRepo(_, Ok(ref repo_score)) | Scores::RepoOnly(ref repo_score) => Some(repo_score),
            Scores::CrateAndRepo(_, Err(_)) | Scores::CrateOnly(_) => None,
        };

        let mut ret = TermString::default();

        if let Some(cr_score) = cr_score_opt {
            let id = cr_score.get_info().get_id();
            ret += self.info_pair(id, false).1 + "\n";

            let table = cr_score.get_score_table();
            ret += EsrPrinter::score_details("Crate Score Details", table) + "\n";
        }

        if let Some(repo_score) = repo_score_opt {
            let table = repo_score.get_score_table();
            ret += EsrPrinter::score_details("Repo Score Details", table) + "\n";

            // Print repo score overview if it wasn't already printed
            if cr_score_opt.is_none() {
                ret += self.score_repo();
            }
        }

        ret
    }

    // =================

    pub async fn collect_scores(crates: &[CrateGeneralInfo], token: &str,
                          crate_only: bool,
                          repo_only: bool) -> Result<Vec<(String, Result<Self>)>> {

        let task_iter = if crate_only {
            crates
                .iter()
                .map(|cr| String::from(cr.get_id()))
                .map(|id| task::spawn(async { (id.clone(), Scores::from_id_crate_only(id).await) }))
                .collect::<Vec<_>>()

        } else if repo_only {
            crates
                .iter()
                .map(|cr| (String::from(cr.get_id()), String::from(token)))
                .map(|(id, token)| task::spawn(async { (id.clone(), Scores::from_id_with_token_repo_only(id, token).await) }))
                .collect::<Vec<_>>()
        } else {
            crates
                .iter()
                .map(|cr| (String::from(cr.get_id()), String::from(token)))
                .map(|(id, token)| task::spawn(async { (id.clone(), Scores::from_id_with_token(id, token).await) }))
                .collect::<Vec<_>>()
        };

        futures::future::join_all(task_iter).await
            .into_iter()
            .map(|res| res.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()
    }

    fn info_pair(&self, id: &str, sort_positive: bool) -> (f64, TermString) {
        match *self {
            Scores::CrateAndRepo(ref cr_score, _) | Scores::CrateOnly(ref cr_score) => {
                let cr_info = cr_score.get_info();
                let (pos, neg) = cr_score.get_score_tuple();

                let empty_or_all_yanked = match cr_info.empty_or_all_yanked() {
                    true => EsrPrinter::all_yanked() + "\n ",
                    false => "\n ".into(),
                };

                let sort_score = match sort_positive {
                    true => pos,
                    false => pos + neg,
                };

                let releases = cr_score.get_score_info().get_releases();
                let non_yanked = cr_score.get_score_info().get_non_yanked_releases();
                let stable = cr_score.get_score_info().get_stable_releases();
                let yanked = releases - non_yanked;
                let non_yanked_pre = non_yanked - stable;
                let releases_formatted = EsrPrinter::releases(stable, non_yanked_pre, yanked);

                let max_ver = Some(cr_info.get_max_version());
                let max_ver_age = cr_info.max_version_age();
                let max_ver_msg = EsrPrinter::release(max_ver, max_ver_age);

                let last_stable_version = cr_info.last_stable_version();
                let last_stable_version_age = cr_info.last_stable_version_age();
                let last_stable_version_msg = EsrPrinter::release(last_stable_version, last_stable_version_age);

                let dependants = cr_score.get_score_info().get_dependants();
                let d_b_n_o = cr_score.get_score_info().get_dependants_from_non_owners();
                let dependants_msg = format!("{} ({} from non owners)", dependants, d_b_n_o);

                let desc = cr_info.get_description()
                    .map(EsrPrinter::desc)
                    .unwrap_or("N/A".into());

                let mut info_formatter = EsrPrinter::id(id) + " ";
                info_formatter += empty_or_all_yanked;
                info_formatter += self.score_crate();
                info_formatter += self.score_repo();
                info_formatter += EsrPrinter::msg_pair("Releases   ", releases_formatted);
                info_formatter += EsrPrinter::msg_pair("Max Version", max_ver_msg);
                info_formatter += EsrPrinter::msg_pair("Last Stable", last_stable_version_msg);
                info_formatter += EsrPrinter::msg_pair("Dependants ", dependants_msg);
                info_formatter += EsrPrinter::msg_pair("License    ", cr_info.get_license().unwrap_or("N/A"));
                info_formatter += EsrPrinter::msg_pair("Repository ", cr_info.get_repository().unwrap_or("N/A"));
                info_formatter += EsrPrinter::msg_pair("Description", desc);

                (sort_score, info_formatter)
            },
            Scores::RepoOnly(ref repo_score) => {
                let (pos, neg) = repo_score.get_score_tuple();
                let sort_score = match sort_positive {
                    true => pos,
                    false => pos + neg,
                };

                let info_formatter = EsrPrinter::id(id) + "\n " + self.score_repo();
                (sort_score, info_formatter)
            },
        }
    }

    pub fn search_results(results: &[(String, Result<Self>)], sort_positive: bool, limit: usize) -> TermString {
        let mut results_vec = Vec::with_capacity(32);
        for res in results {
            match *res {
                (ref id, Ok(ref score_info)) => {
                    results_vec.push(score_info.info_pair(id, sort_positive));
                },
                (ref id, Err(ref e)) => {
                    results_vec.push((f64::MIN, EsrPrinter::err(&format!("{}: Failed to get score info: {}.", id, e)) + "\n"));
                },
            }
        }

        // Negation to get scores in reverse.
        // `* 10000.0` to not lose order accuracy after casting.
        results_vec.sort_by_key(|&(sort_score, _)| -(sort_score * 10000.0) as i64);

        let mut ret = TermString::default();

        for (num, result) in results_vec.iter().take(limit).enumerate() {
            ret += EsrPrinter::id(&format!("({}) ", num + 1));
            ret += result.1.clone() + "\n";
        }

        ret
    }
}
