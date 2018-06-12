/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

macro_rules! score_add {
  ($table:ident, $score:ident, $count:expr, $weight:expr) => {
      {
          let incr = ($count as f64) * $weight;
          let count_str = format!("{: ^49}", stringify!($count).replace("self.", ""));
          let count_mul_weight_num = format!("{: ^18}", format!("{:.3} * {:.3}", $count, $weight));
          let incr_num_str = format!("{:0.3}", incr);
          $table.push((count_str, count_mul_weight_num, incr_num_str));
          $score += incr;
      }
  }
}
