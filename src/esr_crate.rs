/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use hyper::client::Client;
use pipeliner::Pipeline;
use semver::{Version, VersionReq};
use regex::{self, Regex};

use std::collections::HashMap;
use std::result::Result as StdResult;

use esr_errors::*;
use esr_util;
use esr_from::{self, EsrFrom, DefEsrFrom, EsrFromMulti};

#[derive(Deserialize, Debug, Clone)]
pub struct CrateGeneralInfo {
    id: String, // crate name!
    created_at: String,
    updated_at: String,
    max_version: String,
    description: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
    license: Option<String>,
}

impl CrateGeneralInfo {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

#[derive(Deserialize, Debug)]
pub struct CrateReleaseInfo {
    created_at: String,
    downloads: usize,
    num: String, // version
    yanked: bool,
}

#[derive(Deserialize, Debug, Clone)]
struct DependantInfo {
    crate_id: String, // crate name
    default_features: bool,
    optional: bool,
    req: String, // version required
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    id: usize,
}

// =====

#[derive(Deserialize, Debug)]
struct CrateSelfInfo {
    #[serde(rename = "crate")]
    general_info: CrateGeneralInfo,
    #[serde(rename = "versions")]
    releases: Vec<CrateReleaseInfo>,
}

impl EsrFrom for CrateSelfInfo {
    fn url_from_id(id: &str) -> String {
        String::from("https://crates.io/api/v1/crates/") + id
    }
}


#[derive(Deserialize, Debug)]
struct CrateOwners {
    users: Vec<UserInfo>,
}

impl EsrFrom for CrateOwners {
    fn url_from_id(id: &str) -> String {
        let url = String::from("https://crates.io/api/v1/crates/:id/owners");
        url.replace(":id", id)
    }
}

#[derive(Deserialize, Debug)]
struct CrateDependants {
    #[serde(rename = "dependencies")]
    dependants: Vec<DependantInfo>,
    meta: HashMap<String, usize>,
}

impl EsrFromMulti for CrateDependants {
    type Inner = DependantInfo;

    fn get_meta(&self) -> &HashMap<String, usize> {
        &self.meta
    }

    fn get_inner(&self) -> &Vec<Self::Inner> {
        &self.dependants
    }

    fn get_inner_mut(&mut self) -> &mut Vec<Self::Inner> {
        &mut self.dependants
    }
}

impl EsrFrom for CrateDependants {
    fn url_from_id(id: &str) -> String {
        let url = String::from("https://crates.io/api/v1/crates/:\
                                id/reverse_dependencies?per_page=100");
        url.replace(":id", id)
    }

    fn from_id(id: &str) -> Result<Self> {
        EsrFromMulti::from_url_multi(&Self::url_from_id(id), true)
    }

    fn from_id_with_client(id: &str, client: &Client) -> Result<Self> {
        EsrFromMulti::from_url_multi_with_init_client(&Self::url_from_id(id), client, true)
    }
}

pub struct CrateInfo {
    self_info: CrateSelfInfo,
    owners: CrateOwners,
    dependants: CrateDependants,
}

impl CrateInfo {
    pub fn get_id(&self) -> &str {
        &self.self_info.general_info.id
    }

    pub fn get_max_version(&self) -> &str {
        &self.self_info.general_info.max_version
    }

    pub fn all_releases(&self) -> &[CrateReleaseInfo] {
        &self.self_info.releases
    }

    pub fn non_yanked_releases(&self) -> Vec<&CrateReleaseInfo> {
        self.self_info
            .releases
            .iter()
            .filter(|release| !release.yanked)
            .collect()
    }

    pub fn stable_releases(&self) -> Vec<&CrateReleaseInfo> {
        self.non_yanked_releases()
            .iter()
            .map(|release| *release)
            .filter(|release| {
                if let Ok(ver) = Version::parse(&release.num) {
                    !ver.is_prerelease()
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn max_version_age(&self) -> Option<Result<f64>> {
        let general_info = &self.self_info.general_info;
        self.all_releases().iter()
            .filter(|r| &r.num == &general_info.max_version).nth(0)
            .map(|r| esr_util::age_in_months(&r.created_at))
    }

    pub fn last_stable_version(&self) -> Option<&str> {
        self.stable_releases().get(0).map(|r| &*r.num)
    }

    pub fn last_stable_version_age(&self) -> Option<Result<f64>> {
        self.stable_releases().get(0).map(|r| esr_util::age_in_months(&r.created_at))
    }

    pub fn empty_or_all_yanked(&self) -> bool {
        let no_releases = self.self_info.releases.is_empty();
        let empty_release = self.get_max_version() == "0.0.0";
        let all_yanked = self.self_info
            .releases
            .iter()
            .find(|release| !release.yanked)
            .is_none();

        no_releases || empty_release || all_yanked
    }

    pub fn get_current_versions(&self) -> Result<Vec<&str>> {
        let self_info = &self.self_info;
        let mut current_versions = Vec::with_capacity(8);

        // max_ver
        // XXX: max_version can point to a yanked version ATM.
        // this may change in the future as it's probably a bug.
        current_versions.push(&*self_info.general_info.max_version);

        // Only take non-yanked releases into account
        let non_yanked_releases = self.non_yanked_releases();

        // last non-yanked, last stable
        current_versions.extend(non_yanked_releases.get(0).map(|r| &*r.num).iter());
        current_versions.extend(self.stable_releases().get(0).map(|r| &*r.num).iter());

        // All releases in the last 30.5 days
        for release in &non_yanked_releases {
            if esr_util::age_in_months(&release.created_at)? <= 1.0 {
                current_versions.push(&*release.num);
            } else {
                break;
            }
        }

        current_versions.sort();
        current_versions.dedup();
        Ok(current_versions)
    }

    pub fn get_description(&self) -> Option<&str> {
        self.self_info
            .general_info
            .description
            .as_ref()
            .map(|s| s.as_str())
    }

    pub fn get_repository(&self) -> Option<&str> {
        self.self_info
            .general_info
            .repository
            .as_ref()
            .map(|s| s.as_str())
    }

    pub fn get_documentation(&self) -> Option<&str> {
        self.self_info
            .general_info
            .documentation
            .as_ref()
            .map(|s| s.as_str())
    }

    pub fn get_license(&self) -> Option<&str> {
        self.self_info
            .general_info
            .license
            .as_ref()
            .map(|s| s.as_str())
    }

    fn github_re() -> StdResult<&'static Regex, &'static regex::Error> {
        lazy_static! {
            static ref RE: StdResult<Regex, regex::Error> =
                Regex::new(r".+://github.com/(.+?/.+?)(.git|/|$).*");
        }
        RE.as_ref()
    }

    pub fn github_id(&self) -> Option<String> {
        let repo_opt = &self.self_info.general_info.repository;
        let re_res = Self::github_re();

        match (repo_opt, re_res) {
            (&Some(ref repo), Ok(re)) => {
                match re.captures(repo) {
                    Some(ref cap) if cap.len() >= 2 => Some(String::from(&cap[1])),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn from_id(id: &str) -> Result<Self> {
        let client = esr_from::mk_client()?;

        Ok(Self {
            self_info: CrateSelfInfo::from_id_with_client(id, &client)?,
            owners: CrateOwners::from_id_with_client(id, &client)?,
            dependants: CrateDependants::from_id_with_client(id, &client)?,
        })
    }

    // The things we do for performance
    pub fn from_id_threaded(id: &str) -> Result<Self> {
        // We use identifiers with urls because it's not guaranteed
        // to collect items below in order when we use `.with_threads()`
        let urls = vec![
            ("self", CrateSelfInfo::url_from_id(id)),
            ("owners", CrateOwners::url_from_id(id)),
            // XXX: We can't add dependants because it requires multi-page.
            // Adding it will silently get results from a single page.
            //("dependants", CrateDependants::url_from_id(id)),
        ];

        let bytes_res: Result<Vec<_>> = urls
            .into_iter()
            .with_threads(2)
            .map(|(ident, url)| DefEsrFrom::bytes_from_url(&url).map(|bytes| (ident, bytes)))
            .collect();

        let bytes = bytes_res?;

        let bytes_self = bytes
            .iter()
            .find(|&&(ident, _)| ident == "self")
            .map(|&(_, ref bytes)| bytes)
            .ok_or("impossible")?;
        let bytes_owners = bytes
            .iter()
            .find(|&&(ident, _)| ident == "owners")
            .map(|&(_, ref bytes)| bytes)
            .ok_or("impossible")?;

        Ok(Self {
            self_info: CrateSelfInfo::from_bytes(bytes_self)?,
            owners: CrateOwners::from_bytes(bytes_owners)?,
            dependants: CrateDependants::from_id(id)?,
        })
    }
}

pub struct CrateScoreInfo {
    // +ve
    has_desc: usize,
    has_docs: usize,
    has_license: usize,
    activity_span_in_months: f64,
    releases: usize,
    non_yanked_releases: usize,
    stable_releases: usize,
    last_2_non_yanked_releases_downloads: usize,
    dependants: usize,
    hard_dependants: usize,
    dependants_on_current_versions: usize,
    dependants_from_non_owners: usize,
    // -ve
    months_since_last_release: f64,
    empty_or_all_yanked: usize,
}

impl CrateScoreInfo {
    fn from_crate_info(crate_info: &CrateInfo) -> Result<Self> {
        let general_info = &crate_info.self_info.general_info;

        let has_desc = general_info.description.is_some() as usize;
        let has_docs = general_info.documentation.is_some() as usize;
        let has_license = general_info.license.is_some() as usize;
        let empty_or_all_yanked = crate_info.empty_or_all_yanked() as usize;

        let releases = crate_info.all_releases().len();
        let non_yanked_releases = crate_info.non_yanked_releases().len();
        let stable_releases = crate_info.stable_releases().len();
        let last_2_non_yanked_releases_downloads = crate_info
            .non_yanked_releases()
            .iter()
            .take(2)
            .map(|release| release.downloads)
            .sum();

        // time related info
        let activity_span_in_months = esr_util::span_in_months(&general_info.created_at,
                                                               &general_info.updated_at)?;

        let months_since_last_release = match crate_info.non_yanked_releases().get(0) {
            Some(last_release) => esr_util::age_in_months(&last_release.created_at)?,
            None => esr_util::age_in_months(&general_info.created_at)?,
        };

        // Reverse dependencies
        let dependants = crate_info.dependants.dependants.len();

        let current_versions = crate_info.get_current_versions()?;
        let hard_dependants = crate_info.dependants
            .dependants
            .iter()
            .filter(|dependant| dependant.default_features && !dependant.optional)
            .count();
        let dependants_on_current_versions = crate_info.dependants
            .dependants
            .iter()
            .filter(|dependant| {
                current_versions.iter().find(|&ver| {
                    match (Version::parse(ver), VersionReq::parse(&*dependant.req)) {
                        (Ok(ver), Ok(req)) => req.matches(&ver),
                        _ => false,
                    }
                }).is_some()
            })
            .count();

        // We do this in a separate step to make `with_threads()` work
        let owners_ids: Vec<_> = crate_info.owners
            .users
            .iter()
            .map(|user| format!("user_id={}", user.id))
            .collect();

        // Returns first error or a Vec
        // Tip from: https://users.rust-lang.org/t/handling-errors-from-iterators/2551/6
        let owners_crates: Result<Vec<CrateSearch>> = owners_ids
            .into_iter()
            .with_threads(4)
            .map(|id| CrateSearch::from_id(&id))
            .collect();

        // QUIZ: Make this work without this `let`. And without `for`,`while`,etc.
        let owners_crates = owners_crates?;

        let owners_crates_flat: Vec<_> = owners_crates.iter()
            .flat_map(|search| search.crates.iter())
            .collect();

        let dependants_by_owners =
            crate_info.dependants
                .dependants
                .iter()
                .filter_map(|dependant| {
                    owners_crates_flat.iter().find(|cr| cr.id == dependant.crate_id)
                })
                .count();

        let dependants_from_non_owners = dependants - dependants_by_owners;

        Ok(Self {
            // +ve
            has_desc,
            has_docs,
            has_license,
            activity_span_in_months,
            releases,
            non_yanked_releases,
            stable_releases,
            last_2_non_yanked_releases_downloads,
            dependants,
            hard_dependants,
            dependants_on_current_versions,
            dependants_from_non_owners,
            // -ve
            months_since_last_release,
            empty_or_all_yanked,
        })
    }

    fn mk_score(&self) -> (Vec<(String, String, String)>, f64, f64) {
        let mut positive_score = 0.0;
        let mut negative_score = 0.0;
        let mut table = Vec::with_capacity(10);

        // +ve
        score_add!(table, positive_score, self.has_desc, 5.0);
        score_add!(table, positive_score, self.has_license, 5.0);
        score_add!(table, positive_score, self.has_docs, 15.0);

        score_add!(table,
                   positive_score,
                   self.activity_span_in_months.powf(0.5),
                   6.0);

        score_add!(table, positive_score, self.releases, 0.5);
        score_add!(table, positive_score, self.non_yanked_releases, 0.5);
        score_add!(table, positive_score, self.stable_releases, 0.5);
        score_add!(table,
                   positive_score,
                   self.last_2_non_yanked_releases_downloads / 2,
                   0.001);

        score_add!(table, positive_score, self.dependants, 0.5);
        score_add!(table, positive_score, self.hard_dependants, 0.75);
        score_add!(table, positive_score, self.dependants_on_current_versions, 0.75);
        score_add!(table, positive_score, self.dependants_from_non_owners, 2.5);

        // -ve
        score_add!(table,
                   negative_score,
                   self.months_since_last_release.powf(1.5),
                   -2.0);
        score_add!(table, negative_score, self.empty_or_all_yanked, -5000.0);

        (table, positive_score, negative_score)
    }

    pub fn get_dependants(&self) -> usize {
        self.dependants
    }

    pub fn get_dependants_from_non_owners(&self) -> usize {
        self.dependants_from_non_owners
    }

    pub fn get_releases(&self) -> usize {
        self.releases
    }

    pub fn get_non_yanked_releases(&self) -> usize {
        self.non_yanked_releases
    }

    pub fn get_stable_releases(&self) -> usize {
        self.stable_releases
    }

    pub fn get_months_since_last_release(&self) -> f64 {
        self.months_since_last_release
    }
}

// ==============

pub struct CrateInfoWithScore {
    crate_info: CrateInfo,
    crate_score_info: CrateScoreInfo,
    score_positive: f64,
    score_negative: f64,
    score_table: Vec<(String, String, String)>,
}

impl CrateInfoWithScore {
    pub fn from_id(id: &str) -> Result<Self> {
        let crate_info = CrateInfo::from_id_threaded(id)?;
        let crate_score_info = CrateScoreInfo::from_crate_info(&crate_info)?;
        let (score_table, score_positive, score_negative) = crate_score_info.mk_score();

        Ok(Self {
            crate_info,
            crate_score_info,
            score_positive,
            score_negative,
            score_table,
        })
    }

    pub fn get_info(&self) -> &CrateInfo {
        &self.crate_info
    }

    pub fn get_score_info(&self) -> &CrateScoreInfo {
        &self.crate_score_info
    }

    pub fn get_score_tuple(&self) -> (f64, f64) {
        (self.score_positive, self.score_negative)
    }

    pub fn get_score_table(&self) -> &[(String, String, String)] {
        &self.score_table
    }
}

// ==============

#[derive(Deserialize, Debug)]
pub struct CrateSearch {
    crates: Vec<CrateGeneralInfo>,
    meta: HashMap<String, usize>,
}

impl EsrFromMulti for CrateSearch {
    type Inner = CrateGeneralInfo;

    fn get_meta(&self) -> &HashMap<String, usize> {
        &self.meta
    }

    fn get_inner(&self) -> &Vec<Self::Inner> {
        &self.crates
    }

    fn get_inner_mut(&mut self) -> &mut Vec<Self::Inner> {
        &mut self.crates
    }
}

impl EsrFrom for CrateSearch {
    // id here is all search params
    fn url_from_id(id: &str) -> String {
        String::from("https://crates.io/api/v1/crates?per_page=100&") + id
    }

    fn from_id(id: &str) -> Result<Self> {
        EsrFromMulti::from_url_multi(&Self::url_from_id(id), true)
    }

    fn from_id_with_client(id: &str, client: &Client) -> Result<Self> {
        EsrFromMulti::from_url_multi_with_init_client(&Self::url_from_id(id), client, true)
    }
}

impl CrateSearch {
    // id here is all search params
    pub fn from_id_single_page(id: &str) -> Result<Self> {
        let mut url = String::from("https://crates.io/api/v1/crates?");
        url.push_str(id);
        EsrFromMulti::from_url_multi(&url, false)
    }

    pub fn get_crates(&self) -> &[CrateGeneralInfo] {
        &self.crates
    }
}
