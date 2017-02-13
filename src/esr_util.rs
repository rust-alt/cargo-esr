use time;

use esr_errors::*;

fn date_sec(date: &str) -> Result<f64> {
    let date_tm = time::strptime(date, "%FT%TZ")?;
    Ok(date_tm.to_timespec().sec as f64)
}

pub fn age_in_months(date: &str) -> Result<f64> {
    let date = date_sec(date)?;
    let curr_date = time::get_time().sec as f64;
    let age = (curr_date - date) / (3600.0 * 24.0 * 30.5);
    Ok(age)
}

pub fn span_in_months(date1: &str, date2: &str) -> Result<f64> {
    let date1 = date_sec(date1)?;
    let date2 = date_sec(date2)?;
    let span = (date2 - date1).abs() / (3600.0 * 24.0 * 30.5);
    Ok(span)
}
