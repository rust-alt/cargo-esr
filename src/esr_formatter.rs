/*
    This file is a part of cargo-esr.

    Copyright (C) 2017 Mohammad AlSaleh <CE.Mohammad.AlSaleh at gmail.com>
    https://github.com/rust-alt/cargo-esr

    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
*/

use term::{self, Attr};

use std::borrow::Borrow;
use std::ops::{Add, AddAssign, BitOr, BitOrAssign};
use std::mem;

use esr_errors::Result;

#[derive(Clone)]
pub struct EsrFmtAttrs {
    attrs: [Option<Attr>; 11],
}

impl EsrFmtAttrs {
    pub fn empty() -> Self {
        Self { attrs: [None; 11], }
    }

    pub fn is_empty(&self) -> bool {
        self.attrs.iter().find(|&attr| attr.is_some()).is_none()
    }

    fn _append_attr_unchecked(&mut self, attr: Attr) {
        let first_none = self.attrs.iter()
            .position(|&g_attr| g_attr == None)
            .expect("should never happen");

        self.attrs[first_none] = Some(attr);
    }

    fn _append_attr(&mut self, attr: Attr, safe: bool) {
        match attr {
            Attr::Bold | Attr::Dim | Attr::Blink | Attr::Reverse | Attr::Secure => {
                if self.attrs.iter().find(|&&g_attr| g_attr == Some(attr)).is_none() {
                    self._append_attr_unchecked(attr);
                }
            },

            Attr::Italic(_) | Attr::Underline(_) | Attr::Standout(_) | Attr::ForegroundColor(_) | Attr::BackgroundColor(_) => {
                let variant_match_opt = self.attrs.iter()
                    .filter_map(|&g_attr| g_attr)
                    .find(|g_attr| mem::discriminant(g_attr) == mem::discriminant(&attr));

                match variant_match_opt {
                    None => self._append_attr_unchecked(attr),
                    Some(variant_match) => {
                        if variant_match != attr && !safe {
                            let pos_to_replace = self.attrs.iter()
                                .position(|&g_attr| g_attr == Some(variant_match))
                                .expect("should never happen");
                            self.attrs[pos_to_replace] = Some(attr);
                        }
                    },
                };
            },
        };
    }

    fn append_attr(&mut self, attr: Attr) {
        // safe = false
        self._append_attr(attr, false);
    }

    fn append_attr_safe(&mut self, attr: Attr) {
        // safe = true
        self._append_attr(attr, true);
    }

    pub fn append_attrs(&mut self, attrs: &impl Borrow<[Attr]>) {
        attrs.borrow().iter().for_each(|&attr| self.append_attr(attr));
    }

    pub fn append_attrs_safe(&mut self, attrs: &impl Borrow<[Attr]>) {
        attrs.borrow().iter().for_each(|&attr| self.append_attr_safe(attr));
    }

    pub fn with_appended_attrs(mut self, attrs: &impl Borrow<[Attr]>) -> Self {
        self.append_attrs(attrs);
        self
    }

    pub fn with_appended_attrs_safe(mut self, attrs: &impl Borrow<[Attr]>) -> Self {
        self.append_attrs_safe(attrs);
        self
    }

    pub fn merge(&mut self, other: &Self) {
        other.attrs.iter()
            .filter_map(|&attr| attr)
            .for_each(|attr| self.append_attr(attr));
    }
}

impl<A: Borrow<[Attr]>> From<A> for EsrFmtAttrs {
    fn from(attrs: A) -> Self {
        Self::empty().with_appended_attrs(&attrs)
    }
}

impl BitOr<Attr> for EsrFmtAttrs {
    type Output = Self;
    fn bitor(self, attr: Attr) -> Self {
        self.with_appended_attrs(&[attr])
    }
}

impl BitOrAssign<Attr> for EsrFmtAttrs {
    fn bitor_assign(&mut self, attr: Attr) {
        self.append_attrs(&[attr]);
    }
}

#[derive(Clone)]
pub struct EsrFormatter {
    style: EsrFmtAttrs,
    text: String,
    tail: Vec<EsrFormatter>,
}

impl EsrFormatter {
    pub fn new(style: EsrFmtAttrs, text: &str) -> Self {
        Self {
            style,
            text: String::from(text),
            tail: Vec::with_capacity(32),
        }
    }

    pub fn len(&self) -> usize {
        let mut ret = self.text.len();
        self.tail.iter().for_each(|f| ret += f.len());
        ret
    }

    pub fn as_sting(&self) -> String {
        let mut ret = self.text.clone();
        self.tail.iter().for_each(|f| ret += &f.as_sting());
        ret
    }

    pub fn append_style(&mut self, attrs: &impl Borrow<[Attr]>) {
        self.style.append_attrs(attrs);
        self.tail.iter_mut().for_each(|f| f.style.append_attrs(attrs));
    }

    pub fn with_appended_style(mut self, attrs: &impl Borrow<[Attr]>) -> Self {
        self.style.append_attrs(attrs);
        self
    }

    pub fn with_merged_style(mut self, style: &EsrFmtAttrs) -> Self {
        self.style.merge(style);
        self.tail.iter_mut().for_each(|f| f.style.merge(style));
        self
    }

    pub fn merge_style(&mut self, style: &EsrFmtAttrs) {
        self.style.merge(style);
        self.tail.iter_mut().for_each(|f| f.style.merge(style));
    }

    pub fn reset_tail(&mut self) {
        self.tail = Vec::new();
    }

    pub fn set_tail(&mut self, tail: &impl Borrow<[Self]>) {
        self.tail = Vec::from(tail.borrow())
    }

    pub fn extend_tail(&mut self, tail: &impl Borrow<[Self]>) {
        self.tail.extend_from_slice(tail.borrow());
    }

    pub fn push_tail(&mut self, tail: impl Into<Self>) {
        self.tail.push(tail.into());
    }

    pub fn with_tail(mut self, tail: &impl Borrow<[Self]>) -> Self {
        self.set_tail(tail);
        self
    }

    pub fn with_reset_tail(mut self) -> Self {
        self.reset_tail();
        self
    }

    pub fn with_extended_tail(mut self, tail: &impl Borrow<[Self]>) -> Self {
        self.tail.extend_from_slice(tail.borrow());
        self
    }

    fn print_with_trail(&self, formatted: bool, trail: Option<&str>) -> Result<()> {
        match (formatted, term::stdout()) {
            (true, Some(mut out)) => {
                // It's important to reset so text with None style does not inherit attrs
                out.reset()?;

                for attr in self.style.attrs.iter().filter_map(|&attr| attr) {
                    out.attr(attr)?;
                }

                write!(out, "{}", self.text)?;
            },
            _ => print!("{}", &self.text),
        };

        for t in &self.tail {
            t.print(formatted);
        }

        if let Some(trail) = trail {
            let trail: Self = trail.into();
            trail.print(formatted);
        }
        Ok(())
    }

    pub fn print(&self, formatted: bool) {
        self.print_with_trail(formatted, None).expect("Printer failed. Something is wrong.");
    }

    pub fn println(&self, formatted: bool) {
        self.print_with_trail(formatted, Some("\n")).expect("Printer failed. Something is wrong.");
    }
}

impl<B :Borrow<str>> From<B> for EsrFormatter {
    fn from(s: B) -> Self {
        Self::new(EsrFmtAttrs::empty(), s.borrow())
    }
}

impl<I: Into<Self> + Clone> Add<I> for EsrFormatter {
    type Output = Self;
    fn add(self, other: I) -> Self {
        let mut s = self;
        let other_without_tail = other.clone().into().with_reset_tail();
        s.push_tail(other_without_tail);
        s.extend_tail(&other.into().tail);
        s
    }
}


impl<I: Into<Self> + Clone> AddAssign<I> for EsrFormatter {
    fn add_assign(&mut self, other: I) {
        let other_without_tail = other.clone().into().with_reset_tail();
        self.push_tail(other_without_tail);
        self.extend_tail(&other.into().tail);
    }
}
