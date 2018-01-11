# cargo-esr: Extended Search & Ranking

 `cargo-esr` is a proof-of-concept tool that uses the de facto
 [crates.io](https://crates.io) API to search for crates, and 
 then rank the results based on *measured* relevance.

 Additionally, a repository score is generated. But not taken into consideration
 when sorting search results. Only GitHub repositories are supported.

 Score contributing factors, and the chosen weight for them is completely
 arbitrary. And thus shouldn't be taken too seriously. Neither should
 the **exact scores** be relied on for evaluation (more on that below in
 the **Caveats** section).

 The idea is to try to narrow down the possibilities to 2-3 solid choices.
 Reducing the need for constantly engaging with the *official community*.
 And providing a more objective view into the swarm, and its current state
 of affairs.

## Contact

 Feel free to ask or propose anything in the meta [Questions & Chat](https://github.com/rust-alt/cargo-esr/issues/1) issue.

## Install & Usage

### Install

 A nightly compiler is required.

 ```
 git clone https://github.com/rust-alt/cargo-esr.git
 cd cargo-esr
 cargo install
 ```

 Now you can use `esr` as a `cargo` subcommand.

### Usage

 ```
 $ cargo esr -h
 ```

 > Getting repository information requires passing
 > a [GitHub access token](https://github.com/settings/tokens/new)
 > with `-t <token>`. Or setting `CARGO_ESR_GH_TOKEN` in the environment.
 >
 > This is required to avoid hitting rate-limits enforced by GitHub.
 >
 > Alternatively, passing `-C` will skip getting repository scores.
 >

## Detailed Scoring Criteria

 Let's take `mio`'s score as an example:
 ```
 $ cargo esr -c mio
  mio
  Crate Score: 615.475 (+615.630 / -0.155)
  Repo Score : 791.627 (+791.733 / -0.106)
  Releases   : 30+0+0
  Max Version: 0.6.12 (released 0.2 months ago)
  Last Stable: 0.6.12 (released 0.2 months ago)
  Dependants : 118 (106 from non owners)
  License    : MIT
  Repository : https://github.com/carllerche/mio
  Description: Lightweight non-blocking IO
 
 -------------------------------------------------
                Crate Score Details
 -------------------------------------------------
                     has_desc                      |     1 * 5.000      | +5.000
                    has_license                    |     1 * 5.000      | +5.000
                     has_docs                      |     1 * 15.000     | +15.000
         activity_span_in_months.powf(0.5)         |   6.143 * 6.000    | +36.859
                     releases                      |     30 * 0.500     | +15.000
                non_yanked_releases                |     30 * 0.500     | +15.000
                  stable_releases                  |     30 * 0.500     | +15.000
  last_2_non_yanked_releases_downloads.powf(0.5)   |  325.209 * 0.100   | +32.521
                    dependants                     |    118 * 0.500     | +59.000
                  hard_dependants                  |    113 * 0.750     | +84.750
          dependants_on_current_versions           |     90 * 0.750     | +67.500
            dependants_from_non_owners             |    106 * 2.500     | +265.000
        months_since_last_release.powf(1.5)        |   0.078 * -2.000   | -0.155
                empty_or_all_yanked                |   0 * -5000.000    | 0.000
 
 Crate Score: 615.475 (+615.630 / -0.155)
 
 -------------------------------------------------
                Repo Score Details
 -------------------------------------------------
                    subscribers                    |    102 * 0.500     | +51.000
              contributors_up_to_100               |    100 * 3.000     | +300.000
        commits_from_upto_100_contributors         |    617 * 0.050     | +30.850
            secondary_contribution_pct             |     50 * 2.000     | +100.000
             tertiary_contribution_pct             |     40 * 3.000     | +120.000
           push_span_in_months.powf(0.5)           |   6.377 * 5.000    | +31.883
         merged_pull_requests_in_last_100          |     79 * 2.000     | +158.000
       months_since_last_pr_merged.powf(1.5)       |   0.078 * -1.000   | -0.078
     months_since_last_issue_closed.powf(1.5)      |   0.027 * -1.000   | -0.027
         months_since_last_push.powf(1.5)          |   0.000 * -4.000   | -0.001
 
 Repo Score : 791.627 (+791.733 / -0.106)

 ```

 The first column shows the score contributor factors. The 2nd column shows
 the factors' values multiplied by the chosen weights for each one of them.
 The 3rd column shows the result of the multiplication.

 Negative scores are indicators of inactivity.

 A short explanation for each contributing factor follows:

### Crate Score

#### has_desc
   The crate has a description.

#### has_license
   The crate has a license.

#### has_docs
   The crate has documentation.

   That's just a URL the author sets. It doesn't speak to
   the quality or the completeness of the documentation.

#### has_activity_span_in_months.powf(0.5)
   The span from crate's creation date on [crates.io](https://crates.io)
   until the last update.

   Non-linear because we want to limit the reward as crates grow older.

#### releases
   The number of releases the crate has.

#### non_yanked_releases
   The number of non-yanked releases the crate has.

#### stable_releases
   The number of non-yanked non-pre releases the crate has.

#### last_2_non_yanked_releases_downloads.powf(0.5)
   The total number of downloads of the last two non-yanked releases.

   Non-linear because we want to limit the effect a huge number of downloads can
   have on the total score.

   A huge number of downloads for a seemingly-unpopular crate is not necessarily
   a faked stat. Some crates were dependencies of one or more popular crates,
   but they are not anymore.

   This factor will be adjusted if date-based download stats ever become available.

#### dependants
   The number of dependants (a.k.a. reverse dependencies).

#### hard_dependants
   The number of dependants that non-optionally depend on this crate in their default feature.

#### dependants_on_current_versions
   The number of dependants that depend on a version of this crate that
   is SemVer-compatible with one or more of the following:

   * max_version.
   * The version of the last non-yanked release.
   * The version of the last stable release.
   * Any non-yanked version that has been released in the last 30.5 days.

#### dependants_from_non_owners
   The number of dependants from other authors than the authors of this
   crate.

   This is probably the most relevant factor. And it is indeed the reason
   behind the good results you get at the top when you use this tool.

   It speaks to the popularity and usability of the crate by others. It
   also perfectly reflects the current state of affairs.
   It tells us, for example, whether people actually moved en masse from
   one popular , but arguably deprecated, crate to another. It's
   the anti-anecdote factor, of sorts.

#### empty_or_all_yanked
   Whether the crate has no releases, or max_version is `0.0.0`, or all releases
   of the crate have been yanked.

   This is a strong negative factor (-5000.0), with an additional indicator in
   the search results displayed.

#### months_since_last_release.powf(1.5)
   The number of months (floating point) since the last non-yanked version
   released.

   This is a negative factor.

   Non-linear because the longer the crate is inactive, the more we want to punish it.


### Repo Score
#### subscribers
   The number of subscribers/watchers of the repo.

#### contributors_up_to_100
   The number of contributors to the repo. Up to a maximum of a 100.

#### commits_from_upto_100_contributors
   The number of commits pushed to the repo, from up to 100 contributors.

#### secondary_contribution_pct
   For repositories with 50 or more commits. This represents the percentage
   of commits from all contributors but the top one.

#### tertiary_contribution_pct
   For repositories with 50 or more commits. This represents the percentage
   of commits from all contributors but the top two.

   This, with the number of contributors, provide an alternative to bus/truck
   factor calculations, from readily available data, obtained via GitHub's API.

#### push_span_in_months.powf(0.5)
   The span in months (floating point) from the repository's creation, to
   the last push.

   Pushes to non-default branches are taken into account.

#### merged_pull_requests_in_last_100
   The number of pull requests merged in the last 100 PRs sent to the repository.
   This will be the number of all PRs merged in smaller repositories.

#### months_since_last_pr_merged.powf(1.5)
   The number of months (floating point) since the last pull request merged.
   This will be the number of months since the repository was created, if
   it never had a PR merged.

   This is a negative factor.

   Non-linear because the longer the repository is inactive, the more we want to punish it.

#### months_since_last_issue_closed.powf(1.5)
   The number of months (floating point) since the last issue closed.
   This will be the number of months since the repository was created, if
   it never had an issue closed.

   This is a negative factor.

   Non-linear because the longer the repository is inactive, the more we want to punish it.

#### months_since_last_push.powf(1.5)
   The number of months (floating point) since the last push to the repository.
   Pushes to non-default branches are taken into account.

   This is the most relevant negative factor. And thus has the highest weight.

   Non-linear because the longer the repository is inactive, the more we want to punish it.

## Caveats

 * The code is horrible. Please don't look. It also lacks tests and comments.

 * The tool relies on a de facto API. And thus can never be considered stable.
   As it can break at any time.

 * GitHub score factors are not the best. It's the best we can get without
   making many API calls. A dedicated service with caching can definitely do better.

 * The weights given to each contributing factor are arbitrary.

 * Many of the scoring factors can be cheated around (or completely faked). Crate owners can point
   to any repository they want. A crate owner can push to a repository from different accounts, faking
   a higher secondary contribution percentage... etc
  
   Having said that, more weight is given to factors that are harder
   to cheat around without getting caught!

 * Repo scores are biased towards multi-crate repositories.

 * The inactivity factors bias against complete, or maintenance-only crates. This will become more
   relevant when the ecosystem matures.

## A Secondary Goal

 Another goal of this tool is to provide a counter view against the effort to
 curate/officiate/bless certain crates, based on the false (IMHO) premise that
 there is no other (objective) way for people outside *the community* to find
 those crates.

 *The swarm* is deciding what crates it wants to use. And it is continually
 adjusting those decisions. And we can easily follow those decisions and
 adjustments.

 Curation/Officiating/Blessing processes also invite their own problems.
 From favouritism to social engineering. And from **Tip Rot(TM)** to maintenance
 fatigue. We have enough experience to know that they don't work that well in
 the long run.

 Now, that doesn't mean that official documentation shouldn't point to crates
 that provide better alternatives than whatever is available in std. People who
 are not actively engaging with the community should definitely be able to know
 about crates like `mio` and `clap`, from official documentation.

 What I'm arguing for is pointing to something dynamic rather than static. And this
 tool is my attempt to inspire the community to put its first step in that direction.
