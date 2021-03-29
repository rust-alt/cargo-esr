/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use crates_index::Index;
use semver::{Version, VersionReq};
use serde::Deserialize;
use async_trait::async_trait;
use futures::future;
use once_cell::sync::OnceCell;

use crate::esr_errors::{Result, EsrError};
use crate::esr_util;
use crate::esr_from::{Meta, EsrFrom, EsrFromMulti};
use crate::esr_printer::EsrPrinter;

fn get_index() -> Result<&'static Index> {
        static INDEX: OnceCell<std::result::Result<Index, String>> = OnceCell::new();
        let init = || {
            EsrPrinter::crate_index_init().eprintln();
            let index = Index::new_cargo_default();
            index.retrieve_or_update()
                .map_err(|e| e.to_string())
                .map(|_| index)
        };
        INDEX.get_or_init(init)
            .as_ref()
            .map_err(|e| EsrError::from(&**e))
}

#[derive(Deserialize, Debug, Clone)]
pub struct CrateGeneralInfo {
    id: String, // crate name!
    created_at: String,
    updated_at: String,
    max_version: String,
    description: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
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
    license: Option<String>,
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    id: usize,
}

// =====

#[derive(Deserialize, Debug)]
pub struct CrateInfo {
    #[serde(rename = "crate")]
    general_info: CrateGeneralInfo,
    #[serde(rename = "versions")]
    releases: Vec<CrateReleaseInfo>,
}

impl EsrFrom for CrateInfo {
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
struct Dependant {
    crate_name: String,
    default_features: bool,
    optional: bool,
    req: String, // version required
}

impl Dependant {
    async fn dependants_from_id(id: String) -> Result<Vec<Self>> {
        log::debug!("Getting dependats from index for {}", id);
        smol::spawn( async move {
            let mut ret = Vec::with_capacity(64);
            for cr in get_index()?.crates() {
                let latest_version = cr.latest_version();
                let match_opt = latest_version
                    .dependencies()
                    .iter()
                    .find(|dep| !latest_version.is_yanked() && dep.crate_name() == id);

                if let Some(dep) = match_opt {
                    ret.push(
                        Self {
                            crate_name: cr.name().into(),
                            default_features: dep.has_default_features() && !dep.is_optional(),
                            optional: dep.is_optional(),
                            req: dep.requirement().into(),
                        }
                    );
                }
            }
            Ok(ret)
        }).await
    }
}

impl CrateInfo {
    pub fn get_id(&self) -> &str {
        &self.general_info.id
    }

    pub fn get_max_version(&self) -> &str {
        &self.general_info.max_version
    }

    pub fn all_releases(&self) -> &[CrateReleaseInfo] {
        &self.releases
    }

    pub fn non_yanked_releases(&self) -> Vec<&CrateReleaseInfo> {
        self
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
        let general_info = &self.general_info;
        self.all_releases().iter()
            .filter(|r| &r.num == &general_info.max_version).nth(0)
            .map(|r| esr_util::age_in_months(&esr_util::crate_to_iso8601(&r.created_at)))
    }

    pub fn last_stable_version(&self) -> Option<&str> {
        self.stable_releases().get(0).map(|r| &*r.num)
    }

    pub fn last_stable_version_age(&self) -> Option<Result<f64>> {
        self.stable_releases().get(0).map(|r| esr_util::age_in_months(&esr_util::crate_to_iso8601(&r.created_at)))
    }

    pub fn empty_or_all_yanked(&self) -> bool {
        let no_releases = self.releases.is_empty();
        let empty_release = self.get_max_version() == "0.0.0";
        let all_yanked = self
            .releases
            .iter()
            .find(|release| !release.yanked)
            .is_none();

        no_releases || empty_release || all_yanked
    }

    pub fn get_current_versions(&self) -> Result<Vec<&str>> {
        let mut current_versions = Vec::with_capacity(8);

        // max_ver
        // XXX: max_version can point to a yanked version ATM.
        // this may change in the future as it's probably a bug.
        current_versions.push(&*self.general_info.max_version);

        // Only take non-yanked releases into account
        let non_yanked_releases = self.non_yanked_releases();

        // last non-yanked, last stable
        current_versions.extend(non_yanked_releases.get(0).map(|r| &*r.num).iter());
        current_versions.extend(self.stable_releases().get(0).map(|r| &*r.num).iter());

        // All releases in the last 30.5 days
        for release in &non_yanked_releases {
            if esr_util::age_in_months(&esr_util::crate_to_iso8601(&release.created_at))? <= 1.0 {
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
        self
            .general_info
            .description
            .as_ref()
            .map(|s| s.as_str())
    }

    pub fn get_repository(&self) -> Option<&str> {
        self
            .general_info
            .repository
            .as_ref()
            .map(|s| s.as_str())
    }

    pub fn get_documentation(&self) -> Option<&str> {
        self
            .general_info
            .documentation
            .as_ref()
            .map(|s| s.as_str())
    }

    pub fn get_license(&self) -> Option<&str> {
        // TODO: re-write when impl Try for Option is implemented
        let max_ver_rel = self.releases
            .iter()
            .find(|rel| rel.num == self.get_max_version());

        match max_ver_rel {
            None => None,
            Some(rel) => rel.license.as_ref().map(|l| l.as_str()),
        }
    }

    pub fn github_id(&self) -> Option<String> {
        match self.general_info.repository {
            Some(ref repo) => esr_util::github_repo(repo),
            None => None,
        }
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
    last_2_non_yanked_releases_downloads: f64,
    dependants: usize,
    hard_dependants: usize,
    dependants_on_current_versions: usize,
    dependants_from_non_owners: usize,
    // -ve
    months_since_last_release: f64,
    empty_or_all_yanked: usize,
}

impl CrateScoreInfo {
    async fn from_crate_info(crate_info: &CrateInfo) -> Result<Self> {
        let general_info = &crate_info.general_info;

        let owners_info_fut = smol::spawn(CrateOwners::from_id_owned(general_info.id.clone()));
        let dependants_info_fut = smol::spawn(Dependant::dependants_from_id(general_info.id.clone()));

        let has_desc = general_info.description.is_some() as usize;
        let has_docs = general_info.documentation.is_some() as usize;
        let has_license = crate_info.get_license().is_some() as usize;
        let empty_or_all_yanked = crate_info.empty_or_all_yanked() as usize;

        let releases = crate_info.all_releases().len();
        let non_yanked_releases = crate_info.non_yanked_releases().len();
        let stable_releases = crate_info.stable_releases().len();
        let last_2_non_yanked_releases_downloads = crate_info
            .non_yanked_releases()
            .iter()
            .take(2)
            .map(|release| release.downloads as f64)
            .sum();

        // time related info
        let activity_span_in_months = esr_util::span_in_months(&esr_util::crate_to_iso8601(&general_info.created_at),
                                                               &esr_util::crate_to_iso8601(&general_info.updated_at))?;

        let months_since_last_release = match crate_info.non_yanked_releases().get(0) {
            Some(last_release) => esr_util::age_in_months(&esr_util::crate_to_iso8601(&last_release.created_at))?,
            None => esr_util::age_in_months(&esr_util::crate_to_iso8601(&general_info.created_at))?,
        };

        let dependants_info = dependants_info_fut.await?;
        let owners_info = owners_info_fut.await?;

        // Reverse dependencies
        let dependants = dependants_info.len();

        let current_versions = crate_info.get_current_versions()?;
        let hard_dependants = dependants_info
            .iter()
            .filter(|dependant| dependant.default_features && !dependant.optional)
            .count();
        let dependants_on_current_versions = dependants_info
            .iter()
            .filter(|dependant| {
                current_versions.iter().any(|&ver| {
                    match (Version::parse(ver), VersionReq::parse(&*dependant.req)) {
                        (Ok(ver), Ok(req)) => req.matches(&ver),
                        _ => false,
                    }
                })
            })
            .count();

        // We do this in a separate step to make `with_threads()` work
        let owners_ids: Vec<_> = owners_info
            .users
            .iter()
            .map(|user| format!("user_id={}", user.id))
            .collect();

        let owners_crates = owners_ids
            .into_iter()
            .map(|id| smol::spawn(CrateSearch::from_id_owned(id)));

        let owners_crates = future::join_all(owners_crates)
            .await
            .into_iter()
            .map(|t| t.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        let owners_crates_flat: Vec<_> = owners_crates.iter()
            .flat_map(|search| search.crates.iter())
            .collect();

        let dependants_by_owners =
            dependants_info
                .iter()
                .filter_map(|dependant| {
                    owners_crates_flat.iter().find(|cr| cr.id == dependant.crate_name)
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
        let mut table = Vec::with_capacity(100);

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
                   self.last_2_non_yanked_releases_downloads.powf(0.5),
                   0.1);

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
    pub async fn from_info(crate_info: CrateInfo) -> Result<Self> {
        let crate_score_info = CrateScoreInfo::from_crate_info(&crate_info).await?;
        let (score_table, score_positive, score_negative) = crate_score_info.mk_score();

        Ok(Self {
            crate_info,
            crate_score_info,
            score_positive,
            score_negative,
            score_table,
        })
    }

    pub async fn from_id(id: String) -> Result<Self> {
        let crate_info = CrateInfo::from_id(&*id).await?;
        Self::from_info(crate_info).await
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
    meta: Meta,
}

impl EsrFromMulti for CrateSearch {
    type Inner = CrateGeneralInfo;

    fn get_meta(&self) -> &Meta {
        &self.meta
    }

    fn get_inner(&self) -> &Vec<Self::Inner> {
        &self.crates
    }

    fn get_inner_mut(&mut self) -> &mut Vec<Self::Inner> {
        &mut self.crates
    }
}

#[async_trait]
impl EsrFrom for CrateSearch {
    // id here is all search params
    fn url_from_id(id: &str) -> String {
        String::from("https://crates.io/api/v1/crates?per_page=100&") + id
    }

    async fn from_id(id: &str) -> Result<Self> {
        EsrFromMulti::from_url_multi(&*Self::url_from_id(id), true).await
    }
}

impl CrateSearch {
    // id here is all search params
    pub async fn from_id_single_page(id: &str) -> Result<Self> {
        let mut url = String::from("https://crates.io/api/v1/crates?");
        url.push_str(id);
        EsrFromMulti::from_url_multi(&*url, false).await
    }

    pub fn get_crates(&self) -> &[CrateGeneralInfo] {
        &self.crates
    }
}
