# cargo-esr: Extended Search & Ranking

**Or:**
 * **Extensive Swarm Review**

**Or:**
 * **Eccentric [Stdx](https://github.com/brson/stdx) Rival**


**Or maybe:**
 * **Extended Support Release**
   The Firefox release channel I had to move to, because Vimperator broke again
   after a Firefox update.

**Or:**
 * whatever acronym expansion that will help you remember the name of this tool ;)

---

![cargo-esr screenshot](https://github.com/rust-alt/cargo-esr-misc/raw/master/cargo-esr-screenshot.png)

 `cargo-esr` is a proof-of-concept tool that uses the de facto
 [crates.io](https://crates.io) API to search for crates, and 
 then rank the results based on *measured* relevance.

 Additionally, a repository score is generated. But not taken into consideration
 when sorting search results. Only GitHub repositories are supported.

 Score contributing factors, and the chosen weight for them is completely
 arbitrary. And thus shouldn't be taken too seriously. Neither should the **exact scores**
 be relied on for evaluation (more on that below in the **Caveats** section).

 The idea is to try to narrow down the possibilities from, let's say 23, to
 2-3 solid choices. Reducing the need for constantly engaging with
 the *official community*. And providing a more objective view into
 the swarm, and its current state of affairs.

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
 cargo-esr 0.2
 cargo's Extended Search & Ranking tool
 
 USAGE:
     cargo-esr [FLAGS] [OPTIONS] <--search <search>...|--score <score>>
 
 FLAGS:
     -o, --crate-only       Get crate info only, without repository scores
     -h, --help             Prints help information
     -n, --no-color         Force disable colors and all formattings
     -p, --sort-positive    Sort by positive scores only. Without taking inactivity
                            into account
     -V, --version          Prints version information
 
 OPTIONS:
     -g, --gh-token <CARGO_ESR_GH_TOKEN>
             Set GitHub Access Token (https://github.com/settings/tokens/new)
     -L, --results-limit <results-limit>
             Limit the number of top-ranking results shown (default: 10, valid: 3-30)
     -l, --search-limit <search-limit>
             Limit the number of searched & ranked items (default: 30, valid: 3-30)
     -c, --score <score>                    Get detailed score of a crate
     -s, --search <search>...
             Search crates.io and return results sorted by score
 ```

 > Getting repository information requires passing
 > a [GitHub access token](https://github.com/settings/tokens/new)
 > with `-g <token>`. Or setting `CARGO_ESR_GH_TOKEN` in the environment.
 >
 > This is required to avoid hitting rate-limits enforced by GitHub.
 >
 > Alternatively, passing `-o` will skip getting repository scores.
 >
 > All below examples assume `CARGO_ESR_GH_TOKEN` is set.

 You can either search (`-s/--search`) `crates.io`. And get results sorted by **crate score**
 in descending order. Or you can get the score details of a crate (`-c/--score`).

 > **Note:** The output in a real terminal emulator is well-formatted with colors.

 Example search:


 ```
 $ cargo esr -s async
 (1) futures
   Crate Score: 435.684 (+436.020 / -0.336)
   Repo Score : 659.578 (+659.578 / -0.000)
   Releases   : 10+1 (0.3 months since last non-yanked release)
   Dependants : 97 (84 from non owners)
   Max Version: 0.1.10
   License    : MIT/Apache-2.0
   Repository : https://github.com/alexcrichton/futures-rs
   Description: An implementation of futures and streams featuring zero allocations,
 composability, and iterator-like interfaces.
 
 (2) mio
   Crate Score: 422.494 (+423.192 / -0.698)
   Repo Score : 642.520 (+642.906 / -0.386)
   Releases   : 22+0 (0.5 months since last non-yanked release)
   Dependants : 85 (75 from non owners)
   Max Version: 0.6.4
   License    : MIT
   Repository : https://github.com/carllerche/mio
   Description: Lightweight non-blocking IO
 
 (3) mioco
   Crate Score: 79.671 (+106.238 / -26.567)
   Repo Score : 268.311 (+307.071 / -38.760)
   Releases   : 13+0 (5.6 months since last non-yanked release)
   Dependants : 2 (1 from non owners)
   Max Version: 0.8.1
   License    : MPL-2.0
   Repository : https://github.com/dpc/mioco
   Description: Scalable, asynchronous IO coroutine-based handling (aka MIO COroutines).
 
 (4) gj
   Crate Score: 67.622 (+126.381 / -58.759)
   Repo Score : -69.039 (+60.865 / -129.903)
   Releases   : 21+0 (9.5 months since last non-yanked release)
   Dependants : 2 (0 from non owners)
   Max Version: 0.2.0
   License    : MIT
   Repository : https://github.com/dwrensha/gj
   Description: Event loop and promises.
 
 (5) pg_async
   Crate Score: 65.791 (+66.032 / -0.241)
   Repo Score : 8.040 (+15.288 / -7.248)
   Releases   : 10+0 (0.2 months since last non-yanked release)
   Dependants : 0 (0 from non owners)
   Max Version: 0.4.1
   License    : MIT
   Repository : https://github.com/ArtemGr/pg_async.rs
   Description: Asynchronous, HA (master-master) PostgreSQL driver on top of libpq
 
 (6) handy_async
   Crate Score: 61.715 (+61.962 / -0.247)
   Repo Score : 8.017 (+20.318 / -12.301)
   Releases   : 7+0 (0.2 months since last non-yanked release)
   Dependants : 2 (0 from non owners)
   Max Version: 0.2.6
   License    : MIT
   Repository : https://github.com/sile/handy_async
   Description: A handy library for describing asynchronous code declaratively
 
 (7) capnp-futures
   Crate Score: 59.654 (+59.917 / -0.263)
   Repo Score : 6.469 (+19.620 / -13.151)
   Releases   : 4+0 (0.3 months since last non-yanked release)
   Dependants : 1 (0 from non owners)
   Max Version: 0.1.1
   License    : MIT
   Repository : https://github.com/dwrensha/capnp-futures-rs
   Description: async serialization for Cap'n Proto messages
 
 (8) amy
   Crate Score: 57.633 (+75.281 / -17.648)
   Repo Score : -12.780 (+40.168 / -52.949)
   Releases   : 15+0 (4.3 months since last non-yanked release)
   Dependants : 1 (0 from non owners)
   Max Version: 0.6.0
   License    : Apache-2.0
   Repository : https://github.com/andrewjstone/amy
   Description: Polling and Registration abstractions around kqueue and epoll for multithreaded async network programming
 
 (9) fibers
   Crate Score: 53.193 (+53.243 / -0.050)
   Repo Score : 8.190 (+15.063 / -6.873)
   Releases   : 4+0 (0.1 months since last non-yanked release)
   Dependants : 1 (0 from non owners)
   Max Version: 0.1.3
   License    : MIT
   Repository : https://github.com/dwango/fibers-rs
   Description: A Rust library to execute a number of lightweight asynchronous tasks (a.k.a, fibers) based on futures and mio
 
 (10) mysql_async
   Crate Score: 52.572 (+53.148 / -0.576)
   Repo Score : 22.468 (+28.740 / -6.273)
   Releases   : 4+0 (0.4 months since last non-yanked release)
   Dependants : 0 (0 from non owners)
   Max Version: 0.5.0
   License    : MIT/Apache-2.0
   Repository : https://github.com/blackbeam/mysql_async
   Description: Tokio based asynchronous MySql client library.
 
 ```

 The default number of searched and ranked crates is 30. You can use
 `-l/--search-limit` to change this default.

 The default number of results shown is 10. You can use `-L/--results-limit`
 to change this default.

 Use `--search-limit` with a lower value if the top results shown are irrelevant.


 Passing `-o/--crate-only` will skip getting repository scores, without affecting
 the order of the results. You will get results faster. And you won't be required
 to use a GitHub access token.

 Now, compare the results we got above here with the results from `cargo search`, which match
 the results [you get on crates.io](https://crates.io/search?q=async+IO):

 ```
 $ cargo search async IO
 handy_async (0.2.5)     A handy library for describing asynchronous code declaratively
 handy_io (0.1.2)        A handy pattern and futures based asynchronous I/O library
 event_rust (0.1.1)      Lightweight non-blocking IO support windows and linux
 tmp_mio (0.5.2)         Temporary fork of the mio crate with windows bug fix
 gio-2-0-sys (0.46.4)    Import crate for Gio
 mio (0.6.4)             Lightweight non-blocking IO
 nio (0.0.1)             Just a stub for upcoming library
 tokio-proto (0.1.0)     A network application framework for rapid development and highly scalable production deployments of clients and servers.
 td_revent (0.1.5)       Event library for Rust, Async IO similar to libevent
 fibers (0.1.1)          A Rust library to execute a number of lightweight asynchronous tasks (a.k.a, fibers) based on futures and mio
 ... and 19 crates more (use --limit N to see more)
 ```

## Detailed Scoring Criteria

 Let's take `mio`'s score as an example:
 ```
 $ cargo esr -c mio
 -------------------------------------------------
                Crate Score Details
 -------------------------------------------------
                   self.has_desc                   |     1 * 5.000      | +5.000
                 self.has_license                  |     1 * 5.000      | +5.000
                   self.has_docs                   |     1 * 15.000     | +15.000
      self.activity_span_in_months.powf(0.5)       |   5.137 * 10.000   | +51.373
                   self.releases                   |     22 * 1.500     | +33.000
             self.non_yanked_releases              |     22 * 1.500     | +33.000
   self.last_2_non_yanked_releases_downloads / 2   |    1693 * 0.001    | +1.693
                  self.dependants                  |     81 * 0.500     | +40.500
     self.hard_dependants_on_current_versions      |     43 * 1.000     | +43.000
          self.dependants_from_non_owners          |     71 * 2.500     | +177.500
     self.months_since_last_release.powf(1.5)      |   0.046 * -2.000   | -0.092
             self.empty_or_all_yanked              |   0 * -5000.000    | 0.000
 
 Crate Score: 404.974 (+405.066 / -0.092)
 -------------------------------------------------
                Repo Score Details
 -------------------------------------------------
                 self.subscribers                  |    115 * 0.500     | +57.500
            self.contributors_up_to_100            |     76 * 3.000     | +228.000
      self.commits_from_upto_100_contributors      |    472 * 0.100     | +47.200
          self.secondary_contribution_pct          |     46 * 5.000     | +230.000
        self.push_span_in_months.powf(0.5)         |   5.408 * 5.000    | +27.038
       self.merged_pull_requests_in_last_100       |     31 * 2.000     | +62.000
    self.months_since_last_pr_merged.powf(1.5)     |   0.046 * -1.000   | -0.046
   self.months_since_last_issue_closed.powf(1.5)   |   0.046 * -1.000   | -0.046
       self.months_since_last_push.powf(1.5)       |   0.003 * -4.000   | -0.013
 
 Repo Score : 651.634 (+651.738 / -0.105)
 ```

 The first column shows the score contributor factors. The 2nd column shows
 the factors' values multiplied by the chosen weights for each one of them.
 The 3rd column shows the result of the multiplication.

 Negative scores are indicators of inactivity.

 A short explanation for each contributing factor follows:

### Crate Score

#### self.has_desc
   The crate has a description.

#### self.has_license
   The crate has a license.

#### self.has_docs
   The crate has documentation.

   That's just a URL the author sets. It doesn't speak to
   the quality or the completeness of the documentation.

#### self.has_self.activity_span_in_months.powf(0.5)
   The span from crate's creation date on [crates.io](https://crates.io)
   until the last update.

   Non-linear because we want to limit the reward as crates grow older.

#### self.releases
   The number of releases the crate has.

#### self.non_yanked_releases
   The number of non-yanked releases the crate has.

#### self.last_2_non_yanked_releases_downloads / 2
   An estimate for the current number of downloads per release.
   This factor has the weakest weight (0.001) among all others.

#### self.dependants
   The number of dependants (a.k.a. reverse dependencies).

#### self.hard_dependants_on_current_versions
   The number of hard dependants on current versions of this crate.

   `hard` means the dependant non-optionally depends on this crate in their default feature.

   `on_current_versions` mean the dependant depends on a version that is
   either `max_ver`, or the last non-yanked one released, or a non-yanked
   version that has been released in the last 30.5 days.

#### self.dependants_from_non_owners
   The number of dependants from other authors than the authors of this
   crate.

   This is probably the most relevant factor. And it is indeed the reason
   behind the good results you get at the top when you use this tool.

   It speaks to the popularity and usability of the crate by others. It
   also perfectly reflects the current state of affairs.
   It tells us, for example, whether people actually moved en masse from
   one popular , but arguably deprecated, crate to another. It's
   the anti-anecdote factor, of sorts.

#### self.empty_or_all_yanked
   Whether the crate has no releases, or max_version is `0.0.0`, or all releases
   of the crate have been yanked.

   This is a strong negative factor (-5000.0), with an additional indicator in
   the search results displayed.

#### self.months_since_last_release.powf(1.5)
   The number of months (floating point) since the last non-yanked version
   released.

   This is a negative factor.

   Non-linear because the longer the crate is inactive, the more we want to punish it.


### Repo Score
#### self.subscribers
   The number of subscribers/watchers of the repo.

#### self.contributors_up_to_100
   The number of contributors to the repo. Up to a maximum of a 100.

#### self.commits_from_upto_100_contributors
   The number of commits pushed to the repo, from up to 100 contributors.

#### self.secondary_contribution_pct
   For repositories with a 100 or more commits. This represents the percentage
   of commits from all contributors but the top one.

   This, with the number of contributors, provide an alternative to bus/truck
   factor calculations, from readily available data, obtained via GitHub's API.

#### self.push_span_in_months.powf(0.5)
   The span in months (floating point) from the repository's creation, to
   the last push.

   Pushes to non-default branches are taken into account.

#### self.merged_pull_requests_in_last_100
   The number of pull requests merged in the last 100 PRs sent to the repository.
   This will be the number of all PRs merged in smaller repositories.

#### self.months_since_last_pr_merged.powf(1.5)
   The number of months (floating point) since the last pull request merged.
   This will be the number of months since the repository was created, if
   it never had a PR merged.

   This is a negative factor.

   Non-linear because the longer the repository is inactive, the more we want to punish it.

#### self.months_since_last_issue_closed.powf(1.5)
   The number of months (floating point) since the last issue closed.
   This will be the number of months since the repository was created, if
   it never had an issue closed.

   This is a negative factor.

   Non-linear because the longer the repository is inactive, the more we want to punish it.

#### self.months_since_last_push.powf(1.5)
   The number of months (floating point) since the last push to the repository.
   Pushes to non-default branches are taken into account.

   This is the most relevant negative factor. And thus has the highest weight.

   Non-linear because the longer the repository is inactive, the more we want to punish it.

## More Examples

 > The searches are done with the default limit(10). Only 3 results are shown for
 > brevity.

### epoll
 
   ```
   $ cargo esr -s epoll
   (1) mio
     Crate Score: 436.043 (+436.077 / -0.034)
     Repo Score : 653.686 (+653.719 / -0.034)
     Dependants : 81 (71 from non owners)
     Releases   : 22 (0.1 months without a release)
     Max Version: 0.6.4
     License    : MIT
     Repository : https://github.com/carllerche/mio
     Description: Lightweight non-blocking IO
   
   (2) epoll
     Crate Score: 111.256 (+113.728 / -2.472)
     Repo Score : -2.619 (+36.206 / -38.826)
     Dependants : 0 (0 from non owners)
     Releases   : 16 (1.2 months without a release)
     Max Version: 2.1.0
     License    : MPL-2.0
     Repository : https://github.com/nathansizemore/epoll
     Description: Safe epoll interface.
   
   (3) amy
     Crate Score: 59.696 (+74.748 / -15.052)
     Repo Score : -4.991 (+40.168 / -45.160)
     Dependants : 1 (0 from non owners)
     Releases   : 15 (3.8 months without a release)
     Max Version: 0.6.0
     License    : Apache-2.0
     Repository : https://github.com/andrewjstone/amy
     Description: Polling and Registration abstractions around kqueue and epoll for multithreaded async network programming
   ```
  
   vs.
  
   ```
   $ cargo search epoll
   epoll (2.1.0)         Safe epoll interface.
   hydrogen (0.1.4)      Multithreaded Linux TCP socket server using epoll.
   amy (0.6.0)           Polling and Registration abstractions around kqueue and epoll for multithreaded async network programming
   reactor (0.1.4)       A wrapper around mio which allows easily composable, but still fast, evented components
   xcore (0.1.3)         A simple epoll based TCP server framework
   cupi (0.1.0)          Cuprum Pi is a GPIO access library written on Rust for the Raspberry Pi board.
   event_rust (0.1.1)    Lightweight non-blocking IO support windows and linux
   gjio (0.1.3)          Asynchronous input and output.
   mio (0.6.4)           Lightweight non-blocking IO
   td_revent (0.1.5)     Event library for Rust, Async IO similar to libevent
   ... and 1 crates more (use --limit N to see more)
   ```
 
### getopt

   ```
   $ cargo esr -s getopt
   (1) clap
     Crate Score: 2217.872 (+2219.003 / -1.131)
     Repo Score : 762.711 (+762.876 / -0.166)
     Dependants : 414 (408 from non owners)
     Releases   : 167 (0.7 months without a release)
     Max Version: 2.20.0
     License    : MIT
     Repository : https://github.com/kbknapp/clap-rs.git
     Description: A simple to use, efficient, and full featured  Command Line Argument Parser
   
   (2) getopts
     Crate Score: 588.292 (+727.888 / -139.596)
     Repo Score : 363.545 (+364.103 / -0.557)
     Dependants : 117 (117 from non owners)
     Releases   : 20 (17.0 months without a release)
     Max Version: 0.2.14
     License    : MIT/Apache-2.0
     Repository : https://github.com/rust-lang/getopts
     Description: getopts-like option parsing.
   
   (3) cargo-edit
     Crate Score: 89.636 (+90.882 / -1.246)
     Repo Score : 472.979 (+473.362 / -0.383)
     Dependants : 2 (2 from non owners)
     Releases   : 7 (0.7 months without a release)
     Max Version: 0.1.6
     License    : Apache-2.0/MIT
     Repository : https://github.com/killercup/cargo-edit
     Description: This extends Cargo to allow you to add and list dependencies by reading/writing to your `Cargo.toml` file from the command line. It contains `cargo add`, `cargo rm`, and `cargo list`.
   ```

   vs.

   ```
   $cargo search getopt
   getopts (0.2.14)      getopts-like option parsing.
   clap (2.20.0)         A simple to use, efficient, and full featured  Command Line Argument Parser
   args (2.0.4)          An argument parsing and validation library designed to take some of tediousness out of the general 'getopts' crate.
   pgetopts (0.1.2)      getopts-like option parsing, a fork of the Rust team's getopts.
   pirate (1.0.0)        A simple arrrguments parser
   rfmt (0.1.0)          Another Rust source code formatter.
   tomllib (0.1.2)       A format-preserving TOML file parser and manipulator
   cargo-edit (0.1.6)    This extends Cargo to allow you to add and list dependencies by reading/writing to your `Cargo.toml` file from the command line…
   du (0.1.1)            Implementing du -sb in order to learn Rust
   glfw-sys (3.2.1)      An Open Source, multi-platform library for creating windows with OpenGL contexts and receiving input and events
   ... and 2 crates more (use --limit N to see more)
   ```

### deserialize

   ```
   $ cargo esr -s deserialize
   (1) serde
     Crate Score: 2741.367 (+2741.379 / -0.012)
     Repo Score : 787.832 (+787.862 / -0.031)
     Dependants : 627 (621 from non owners)
     Releases   : 71 (0.0 months without a release)
     Max Version: 0.9.1
     License    : MIT/Apache-2.0
     Repository : https://github.com/serde-rs/serde
     Description: A generic serialization/deserialization framework
   
   (2) serde_json
     Crate Score: 2367.141 (+2367.147 / -0.006)
     Repo Score : 523.981 (+524.008 / -0.027)
     Dependants : 564 (561 from non owners)
     Releases   : 22 (0.0 months without a release)
     Max Version: 0.9.1
     License    : MIT/Apache-2.0
     Repository : https://github.com/serde-rs/json
     Description: A JSON serialization file format
   
   (3) bincode
     Crate Score: 326.558 (+327.692 / -1.135)
     Repo Score : 342.665 (+346.069 / -3.404)
     Dependants : 36 (33 from non owners)
     Releases   : 35 (0.7 months without a release)
     Max Version: 0.6.1
     License    : MIT
     Repository : https://github.com/TyOverby/bincode
     Description: A binary serialization / deserialization strategy and implementation with serde and rustc-serialize backends.

   ```

   vs.

   ```
   $ cargo search deserialize
   bytevec (0.2.0)               A Rust serialization library that uses byte vectors
   serializable_enum (0.3.1)     Two macros for implementing serialization / deserialization for enums containing no datavariants
   serde_json (0.9.1)            A JSON serialization file format
   serde-redis (0.5.1)           Serde deserialization for redis-rs
   bincode (0.6.1)               A binary serialization / deserialization strategy and implementation with serde and rustc-serialize backends.
   bincode_core (0.6.0)          A binary serialization / deserialization strategy and implementation for serde.
   serde (0.9.1)                 A generic serialization/deserialization framework
   serde_test (0.9.1)            Token De/Serializer for testing De/Serialize implementations
   serde_yaml (0.5.1)            YAML support for Serde
   abomonation_derive (0.1.0)    macros 1.1 derive crate for abomonation
   ... and 51 crates more (use --limit N to see more)
   ```

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
   relevant when the echo system matures.

## A Secondary Goal

 Another goal of this tool is to provide a counter view against the effort to
 curate/officiate/bless certain crates, based on the false (IMHO) premise that
 there is no other (objective) way for people outside *the community* to find
 those crates.

 *The swarm* is deciding what crates it wants to use. And it is continually
 adjusting those decisions. And we can easily follow those decisions and
 adjustments.

 Take `serde` vs. `rustc_serialize` as an example. I personally prefer serde.
 And I use it for deserialization in this very tool.

 *The community* has been talking about `serde` deprecating `rustc_serialize`
 for months. Especially after gaining the ability to deserialize objects with
 unknown fields, and `Macros 1.1` landing in the language. But did the swarm
 move to `serde` en masse already?

 > **Note:** I started writing this section right before `serde`'s 0.9 release.

 As of now, `serde` is a hard dependency of 553 crates. `rustc_serialize`
 is a hard dependency of 750 crates. And rustaceans are hardly known for their
 conservative development tendencies.

 What that tells us is that the majority of rustaceans are content with
 `rustc_serialize`. And have no reason, or feel no urgency, to move
 to `serde`.

 Will the picture be the same in three months? After `Macros 1.1` enjoys
 a few stable releases? And after serde enjoys its next major release?

 We don't know.
 
 But we will know... in three months. Without the need for anyone
 to tell us. And without the need for us to take their word for it.

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
