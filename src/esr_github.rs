/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use pipeliner::Pipeline;

use esr_from::{EsrFrom, DefEsrFrom};
use esr_util;
use esr_errors::Result;

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
    pub fn from_id(_: &str) -> Result<Self> {
        Err("Unimplemented: use  from_id_with_token()")?
    }

    pub fn from_id_with_token(id: &str, token: &str) -> Result<Self> {
        let urls = vec![
            RepoGeneralInfo::url_from_id_and_token(id, token),
            RepoClosedIssues::url_from_id_and_token(id, token),
            RepoPullRequests::url_from_id_and_token(id, token),
            RepoContributors::url_from_id_and_token(id, token),
        ];

        let mut bytes_iter = urls
            .into_iter()
            .with_threads(4)
            .ordered_map(|url| DefEsrFrom::bytes_from_url(&url));

        let bytes_general = bytes_iter.next().expect("impossible")?;
        let bytes_closed_issues = bytes_iter.next().expect("impossible")?;
        let bytes_pulls = bytes_iter.next().expect("impossible")?;
        let bytes_contributors = bytes_iter.next().expect("impossible")?;

        Ok(Self {
            general_info: RepoGeneralInfo::from_bytes(&*bytes_general)?,
            last_100_closed_issues: RepoClosedIssues::from_bytes(&*bytes_closed_issues)?,
            last_100_pull_requests: RepoPullRequests::from_bytes(&*bytes_pulls)?,
            top_100_contributors: RepoContributors::from_bytes(&*bytes_contributors)?,
        })
    }
}

pub struct RepoScoreInfo {
    subscribers: f64,
    contributors_up_to_100: usize,
    commits_from_upto_100_contributors: f64,
    secondary_contribution_pct: usize,
    tertiary_contribution_pct: usize,
    merged_pull_requests_in_last_100: usize,
    months_since_last_pr_merged: f64,
    months_since_last_issue_closed: f64,
    push_span_in_months: f64,
    months_since_last_push: f64,
}

impl RepoScoreInfo {
    fn from_repo_info(repo_info: &RepoInfo) -> Result<Self> {
        let general_info = &repo_info.general_info;

        // Days active, months since last push
        let push_span_in_months =esr_util::span_in_months(&general_info.created_at,
                                                          &general_info.pushed_at)?;
        let months_since_last_push = esr_util::age_in_months(&general_info.pushed_at)?;

        // Get subscribers count
        let subscribers = general_info.subscribers_count as f64;

        // Get contributor count, commit count using contributors info.
        if repo_info.top_100_contributors.is_empty() {
            Err("Empty contributors list")?;
        }

        let contributors_up_to_100 = repo_info.top_100_contributors.len();

        let commits_from_upto_100_contributors = repo_info.top_100_contributors
            .iter()
            .map(|contributor| contributor.contributions)
            .sum::<usize>() as f64;

        // Secondary and Tertiary contribution pct
        let commits = commits_from_upto_100_contributors;
        let top_committer_contrib = repo_info.top_100_contributors[0].contributions as f64 / commits;
        let second_committer_contrib = repo_info.top_100_contributors.get(1)
            .map(|c| c.contributions as f64 / commits)
            .unwrap_or(0_f64);


        let secondary_contribution_pct = ((1.0 - top_committer_contrib) * 100.0).ceil() as usize;
        let tertiary_contribution_pct = ((1.0 - top_committer_contrib - second_committer_contrib) * 100.0).ceil() as usize;

        // merged pull requests in last 100, months since last merged
        let merged_pull_requests_in_last_100 = repo_info.last_100_pull_requests
            .iter()
            .filter(|pr| pr.merged_at.is_some())
            .count();

        let last_pr_merged_opt = repo_info.last_100_pull_requests
            .iter()
            .filter(|pr| pr.merged_at.is_some())
            .nth(0);

        let months_since_last_pr_merged = match last_pr_merged_opt {
            Some(pr) => esr_util::age_in_months(pr.merged_at.as_ref().ok_or("Impossible")?)?,
            None => esr_util::age_in_months(&general_info.created_at)?,
        };

        // months since last closed
        let last_issue_closed_opt = repo_info.last_100_closed_issues.get(0);

        let months_since_last_issue_closed = match last_issue_closed_opt {
            Some(issue) => esr_util::age_in_months(issue.closed_at.as_ref().ok_or("Impossible")?)?,
            None => esr_util::age_in_months(&general_info.created_at)?,
        };

        // Done
        Ok(Self {
            subscribers,
            contributors_up_to_100,
            commits_from_upto_100_contributors,
            secondary_contribution_pct,
            tertiary_contribution_pct,
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
        score_add!(table, positive_score, self.subscribers.powf(0.5), 8.0);
        score_add!(table, positive_score, self.contributors_up_to_100, 3.0);
        score_add!(table,
                   positive_score,
                   self.commits_from_upto_100_contributors.powf(0.5),
                   2.0);

        // We only take secondary/tertiary contribution into account if the repo has >= 50 commits
        if self.commits_from_upto_100_contributors >= 50.0 {
            score_add!(table, positive_score, self.secondary_contribution_pct, 2.5);
            score_add!(table, positive_score, self.tertiary_contribution_pct, 5.0);
        }

        score_add!(table,
                   positive_score,
                   self.push_span_in_months.powf(0.5),
                   5.0);
        score_add!(table,
                   positive_score,
                   self.merged_pull_requests_in_last_100,
                   2.5);

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
    pub fn from_id_with_token(id: &str, token: &str) -> Result<Self> {
        let repo_info = RepoInfo::from_id_with_token(id, token)?;
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
