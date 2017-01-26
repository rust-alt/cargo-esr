/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use time;
use pipeliner::Pipeline;

use esr_from::{self, EsrFrom, DefEsrFrom};
use esr_errors::*;

#[derive(Deserialize, Debug)]
struct PullRequestInfo {
    merged_at: Option<String>,
    number: usize,
}

#[derive(Deserialize, Debug)]
struct IssueInfo {
    closed_at: Option<String>,
    number: usize,
}

#[derive(Deserialize, Debug)]
struct ContributorInfo {
    contributions: usize,
}

// =================

type RepoPullRequests = Vec<PullRequestInfo>;

impl EsrFrom for RepoPullRequests {
    fn url_from_id(id: &str) -> String {
        let url = String::from("https://api.github.com/repos/:id/pulls?state=all&per_page=100");
        url.replace(":id", id)
    }
}

type RepoClosedIssues = Vec<IssueInfo>;

impl EsrFrom for RepoClosedIssues {
    fn url_from_id(id: &str) -> String {
        let url = String::from("https://api.github.com/repos/:id/issues?\
                               state=closed&per_page=1");
        url.replace(":id", id)
    }
}

type RepoContributors = Vec<ContributorInfo>;

impl EsrFrom for RepoContributors {
    fn url_from_id(id: &str) -> String {
        let url = String::from("https://api.github.com/repos/:id/contributors?per_page=100");
        url.replace(":id", id)
    }
}

#[derive(Deserialize, Debug)]
struct RepoGeneralInfo {
    subscribers_count: usize,
    created_at: String,
    // Same as created_at if the repo is empty
    pushed_at: String,
}

impl EsrFrom for RepoGeneralInfo {
    fn url_from_id(id: &str) -> String {
        let url = String::from("https://api.github.com/repos/:id");
        url.replace(":id", id)
    }
}

// =================

pub struct RepoInfo {
    general_info: RepoGeneralInfo,
    last_100_closed_issues: RepoClosedIssues,
    last_100_pull_requests: RepoPullRequests,
    top_100_contributors: RepoContributors,
}

impl RepoInfo {
    pub fn from_id(id: &str) -> Result<Self> {
        let client = esr_from::mk_client()?;

        let info = Self {
            general_info: RepoGeneralInfo::from_id_with_client(id, &client)?,
            last_100_closed_issues: RepoClosedIssues::from_id_with_client(id, &client)?,
            last_100_pull_requests: RepoPullRequests::from_id_with_client(id, &client)?,
            top_100_contributors: RepoContributors::from_id_with_client(id, &client)?,
        };
        Ok(info)
    }

    pub fn from_id_with_token(id: &str, token: &str) -> Result<Self> {
        let client = esr_from::mk_client()?;

        Ok(Self {
            general_info: RepoGeneralInfo::from_id_with_token_and_client(id, token, &client)?,
            last_100_closed_issues:  RepoClosedIssues::from_id_with_token_and_client(id, token, &client)?,
            last_100_pull_requests: RepoPullRequests::from_id_with_token_and_client(id, token, &client)?,
            top_100_contributors: RepoContributors::from_id_with_token_and_client(id, token, &client)?,
        })
    }

    fn urls_from_id(id: &str) -> Vec<(&'static str, String)> {
        // We use identifiers with urls because it's not guaranteed
        // to collect items below in order when we use `.with_threads()`
        vec![
            ("general", RepoGeneralInfo::url_from_id(id)),
            ("issues", RepoClosedIssues::url_from_id(id)),
            ("pulls", RepoPullRequests::url_from_id(id)),
            ("contributors", RepoContributors::url_from_id(id)),
        ]
    }

    fn urls_from_id_with_token(id: &str, token: &str) -> Vec<(&'static str, String)> {
        // We use identifiers with urls because it's not guaranteed
        // to to collect items below in order when we use `.with_threads()`
        vec![
            ("general", RepoGeneralInfo::url_from_id_and_token(id, token)),
            ("issues", RepoClosedIssues::url_from_id_and_token(id, token)),
            ("pulls", RepoPullRequests::url_from_id_and_token(id, token)),
            ("contributors", RepoContributors::url_from_id_and_token(id, token)),
        ]
    }

    // The things we do for performance
    fn from_urls_threaded(urls: Vec<(&'static str, String)>) -> Result<Self> {
        let bytes_res: Result<Vec<_>> = urls
            .into_iter()
            .with_threads(4)
            .map(|(ident, url)| DefEsrFrom::bytes_from_url(&url).map(|bytes| (ident, bytes)))
            .collect();

        let bytes = bytes_res?;

        let bytes_general = bytes
            .iter()
            .find(|&&(ident, _)| ident == "general")
            .map(|&(_, ref bytes)| bytes)
            .ok_or("impossible")?;
        let bytes_issues = bytes
            .iter()
            .find(|&&(ident, _)| ident == "issues")
            .map(|&(_, ref bytes)| bytes)
            .ok_or("impossible")?;
        let bytes_pulls = bytes
            .iter()
            .find(|&&(ident, _)| ident == "pulls")
            .map(|&(_, ref bytes)| bytes)
            .ok_or("impossible")?;
        let bytes_contributors = bytes
            .iter()
            .find(|&&(ident, _)| ident == "contributors")
            .map(|&(_, ref bytes)| bytes)
            .ok_or("impossible")?;

        Ok(Self {
            general_info: RepoGeneralInfo::from_bytes(bytes_general)?,
            last_100_closed_issues: RepoClosedIssues::from_bytes(bytes_issues)?,
            last_100_pull_requests: RepoPullRequests::from_bytes(bytes_pulls)?,
            top_100_contributors: RepoContributors::from_bytes(bytes_contributors)?,
        })
    }

    pub fn from_id_threaded(id: &str) -> Result<Self> {
        let urls = Self::urls_from_id(id);
        Self::from_urls_threaded(urls)
    }

    pub fn from_id_with_token_threaded(id: &str, token: &str) -> Result<Self> {
        let urls = Self::urls_from_id_with_token(id, token);
        Self::from_urls_threaded(urls)
    }
}

pub struct RepoScoreInfo {
    subscribers: usize,
    contributors_up_to_100: usize,
    commits_from_upto_100_contributors: usize,
    secondary_contribution_pct: usize,
    merged_pull_requests_in_last_100: usize,
    months_since_last_pr_merged: f64,
    months_since_last_issue_closed: f64,
    push_span_in_months: f64,
    months_since_last_push: f64,
}

impl RepoScoreInfo {
    fn from_repo_info(repo_info: &RepoInfo) -> Result<Self> {
        let general_info = &repo_info.general_info;
        let curr_time = time::get_time().sec;
        let created_at = time::strptime(&general_info.created_at, "%FT%TZ")?.to_timespec().sec;

        // Days active, months since last push
        let last_push = time::strptime(&general_info.pushed_at, "%FT%TZ")?.to_timespec().sec;
        let push_span_in_months = (last_push - created_at) as f64 / (3600.0 * 24.0 * 30.5);
        let months_since_last_push = (curr_time - last_push) as f64 / (3600.0 * 24.0 * 30.5);

        // Get subscribers count
        let subscribers = general_info.subscribers_count;

        // Get contributor count, commit count using contributors info.
        if repo_info.top_100_contributors.is_empty() {
            Err("Empty contributors list")?;
        }

        let contributors_up_to_100 = repo_info.top_100_contributors.len();

        let commits_from_upto_100_contributors = repo_info.top_100_contributors
            .iter()
            .map(|contributor| contributor.contributions)
            .sum();

        // Secondary contribution pct
        let secondary_contribution_commits = (commits_from_upto_100_contributors -
                                              repo_info.top_100_contributors[0].contributions) as
                                             f64;
        let secondary_contribution_pct_f64 = secondary_contribution_commits * 100.0 /
                                             (commits_from_upto_100_contributors as f64);
        let secondary_contribution_pct = secondary_contribution_pct_f64.ceil() as usize;

        // merged pull requests in last 100, months since last merged
        let mut merged_pull_requests_in_last_100 = 0;
        let mut months_since_last_pr_merged = (curr_time - created_at) as f64 /
                                              (3600.0 * 24.0 * 30.5);

        let last_pr_merged = repo_info.last_100_pull_requests
            .iter()
            .filter(|pr| pr.merged_at.is_some())
            .nth(0);

        if let Some(last_merged) = last_pr_merged {
            let last_merged_at_str = last_merged.merged_at.as_ref().ok_or("Impossible")?;
            let last_merged_at_time = time::strptime(last_merged_at_str, "%FT%TZ")?;
            let last_merged_at = last_merged_at_time.to_timespec().sec;

            months_since_last_pr_merged = (curr_time - last_merged_at) as f64 /
                                          (3600.0 * 24.0 * 30.5);
            merged_pull_requests_in_last_100 = repo_info.last_100_pull_requests
                .iter()
                .filter(|pr| pr.merged_at.is_some())
                .count();
        }

        // months since last closed
        let last_issue_closed = repo_info.last_100_closed_issues.get(0);

        let months_since_last_issue_closed = if let Some(last_closed) = last_issue_closed {
            let last_closed_at_str = last_closed.closed_at.as_ref().ok_or("Impossible")?;
            let last_closed_at_time = time::strptime(last_closed_at_str, "%FT%TZ")?;
            let last_closed_at = last_closed_at_time.to_timespec().sec;

            (curr_time - last_closed_at) as f64 / (3600.0 * 24.0 * 30.5)
        } else {
            (curr_time - created_at) as f64 / (3600.0 * 24.0 * 30.5)
        };

        // Done
        Ok(Self {
            subscribers,
            contributors_up_to_100,
            commits_from_upto_100_contributors,
            secondary_contribution_pct,
            push_span_in_months,
            merged_pull_requests_in_last_100,
            months_since_last_pr_merged,
            months_since_last_issue_closed,
            months_since_last_push,
        })
    }

    fn mk_score(&self) -> (Vec<(String, String, String)>, f64, f64) {
        let mut positive_score = 0.0;
        let mut negative_score = 0.0;
        let mut table = Vec::with_capacity(9);

        // +ve
        score_add!(table, positive_score, self.subscribers, 0.5);
        score_add!(table, positive_score, self.contributors_up_to_100, 3.0);
        score_add!(table,
                   positive_score,
                   self.commits_from_upto_100_contributors,
                   0.1);

        // We only take secondary contribution into account if the repo has >= 100 commits
        if self.commits_from_upto_100_contributors >= 100 {
            score_add!(table, positive_score, self.secondary_contribution_pct, 5.0);
        }

        score_add!(table,
                   positive_score,
                   self.push_span_in_months.powf(0.5),
                   5.0);
        score_add!(table,
                   positive_score,
                   self.merged_pull_requests_in_last_100,
                   2.0);

        // -ve
        score_add!(table,
                   negative_score,
                   self.months_since_last_pr_merged.powf(1.5),
                   -1.0);
        score_add!(table,
                   negative_score,
                   self.months_since_last_issue_closed.powf(1.5),
                   -1.0);
        score_add!(table,
                   negative_score,
                   self.months_since_last_push.powf(1.5),
                   -4.0);

        (table, positive_score, negative_score)
    }
}

pub struct RepoInfoWithScore {
    repo_info: RepoInfo,
    repo_score_info: RepoScoreInfo,
    score_positive: f64,
    score_negative: f64,
    score_table: Vec<(String, String, String)>,
}

impl RepoInfoWithScore {
    pub fn from_id(id: &str) -> Result<Self> {
        let repo_info = RepoInfo::from_id_threaded(id)?;
        let repo_score_info = RepoScoreInfo::from_repo_info(&repo_info)?;
        let (score_table, score_positive, score_negative) = repo_score_info.mk_score();

        Ok(Self {
            repo_info,
            repo_score_info,
            score_positive,
            score_negative,
            score_table,
        })
    }
    pub fn from_id_with_token(id: &str, token: &str) -> Result<Self> {
        let repo_info = RepoInfo::from_id_with_token_threaded(id, token)?;
        let repo_score_info = RepoScoreInfo::from_repo_info(&repo_info)?;
        let (score_table, score_positive, score_negative) = repo_score_info.mk_score();

        Ok(Self {
            repo_info,
            repo_score_info,
            score_positive,
            score_negative,
            score_table,
        })
    }

    pub fn get_info(&self) -> &RepoInfo {
        &self.repo_info
    }

    pub fn get_score_info(&self) -> &RepoScoreInfo {
        &self.repo_score_info
    }

    pub fn get_score_tuple(&self) -> (f64, f64) {
        (self.score_positive, self.score_negative)
    }

    pub fn get_score_table(&self) -> &[(String, String, String)] {
        &self.score_table
    }
}
