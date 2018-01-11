use time;
use regex::{self, Regex};

use esr_errors::*;
use std::result::Result as StdResult;

// Get ISO 8601-formatted string from crate API dates
pub(crate) fn crate_to_iso8601(cr_date: &str) -> String {
    match (cr_date.rfind('.'), cr_date.rfind('+'), cr_date.rfind('Z')) {
        // Do nothing
        (None, _, Some(_))    => String::from(cr_date),
        // Crate format without micro seconds
        (None, Some(i), None) => {
            String::from(&cr_date[0..i]) + "Z"
        },
        // Crate format with micro seconds
        (Some(i), Some(_), None) => String::from(&cr_date[0..i]) + "Z",
        _ => unreachable!(),
    }
}

fn date_sec(date: &str) -> Result<f64> {
    let date_tm = time::strptime(date, "%FT%TZ")?;
    Ok(date_tm.to_timespec().sec as f64)
}

pub(crate) fn age_in_months(date: &str) -> Result<f64> {
    let date = date_sec(date)?;
    let curr_date = time::get_time().sec as f64;
    let age = (curr_date - date) / (3600.0 * 24.0 * 30.5);
    Ok(age)
}

pub(crate) fn span_in_months(date1: &str, date2: &str) -> Result<f64> {
    let date1 = date_sec(date1)?;
    let date2 = date_sec(date2)?;
    let span = (date2 - date1).abs() / (3600.0 * 24.0 * 30.5);
    Ok(span)
}

pub(crate) fn github_re() -> StdResult<&'static Regex, &'static regex::Error> {
    lazy_static! {
        static ref RE: StdResult<Regex, regex::Error> =
            Regex::new(r"(.+://github.com/|@|^)(.+?/.+?)(.git|/|$).*");
    }
    RE.as_ref()
}

pub(crate) fn github_repo(repo: &str) -> Option<String> {
    match github_re() {
        Ok(re) => {
            match re.captures(repo) {
                Some(ref cap) if cap.len() >= 3 => Some(String::from(&cap[2])),
                _ => None,
            }
        },
        _ => None,
    }
}
