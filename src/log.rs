use serde::{Deserialize, Serialize};
use std::{
    cmp, fmt, mem,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Verbose,
    Debug,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Level::Critical => "critical",
                Level::Error => "error",
                Level::Warning => "warning",
                Level::Notice => "notice",
                Level::Info => "info",
                Level::Verbose => "verbose",
                Level::Debug => "debug",
            }
        )
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy, Eq)]
#[repr(usize)]
#[serde(rename_all = "lowercase")]
pub enum LevelFilter {
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Verbose,
    Debug,
}

impl Ord for Level {
    #[inline]
    fn cmp(&self, other: &Level) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}

impl PartialEq for Level {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialEq<LevelFilter> for Level {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd for Level {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn lt(&self, other: &Level) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &Level) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &Level) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &Level) -> bool {
        *self as usize >= *other as usize
    }
}

impl PartialOrd<LevelFilter> for Level {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }

    #[inline]
    fn lt(&self, other: &LevelFilter) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &LevelFilter) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &LevelFilter) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &LevelFilter) -> bool {
        *self as usize >= *other as usize
    }
}

// Below is a slightly modified version of the macros from the `log` crate.

// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

static MAX_LOG_LEVEL_FILTER: AtomicUsize = AtomicUsize::new(0);

#[inline]
pub(crate) fn set_max_level(level: LevelFilter) {
    MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::SeqCst)
}

#[inline(always)]
pub fn max_level() -> LevelFilter {
    // Since `LevelFilter` is `repr(usize)`,
    // this transmute is sound if and only if `MAX_LOG_LEVEL_FILTER`
    // is set to a usize that is a valid discriminant for `LevelFilter`.
    // Since `MAX_LOG_LEVEL_FILTER` is private, the only time it's set
    // is by `set_max_level` above, i.e. by casting a `LevelFilter` to `usize`.
    // So any usize stored in `MAX_LOG_LEVEL_FILTER` is a valid discriminant.
    unsafe { mem::transmute(MAX_LOG_LEVEL_FILTER.load(Ordering::Relaxed)) }
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        let lvl = $lvl;
        if lvl <= $crate::log::max_level() {
            std::println!(
                "log_{}={}",
                lvl, __log_format_args!($($arg)+)
            );
        }
    });
    ($lvl:expr, $($arg:tt)+) => (log!(target: __log_module_path!(), $lvl, $($arg)+))
}

/// Serious errors in protocol or module itself (not in policy)
#[macro_export(local_inner_macros)]
macro_rules! critical {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::log::Level::Critical, $($arg)+)
    );
    ($($arg:tt)+) => (
        log!($crate::Level::Critical, $($arg)+)
    )
}

///  Errors when validating / evaluating a promise, including syntax errors and promise not kept
#[macro_export(local_inner_macros)]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::log::Level::Error, $($arg)+)
    );
    ($($arg:tt)+) => (
        log!($crate::log::Level::Error, $($arg)+)
    )
}

/// The promise did not fail, but there is something the user (policy writer) should probably fix. Some examples:
///
/// * Policy relies on deprecated behavior/syntax which will change
/// * Policy uses demo / unsafe options which should be avoided in a production environment
#[macro_export(local_inner_macros)]
macro_rules! warning {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::log::Level::Warning, $($arg)+)
    );
    ($($arg:tt)+) => (
        log!($crate::log::Level::Warning, $($arg)+)
    )
}

/// Unusual events which you want to notify the user about:
///
/// * Most promise types won't need this - usually info or warning is more appropriate
/// * Useful for events which happen rarely and are not the result of a promise, for example:
///    * New credentials detected
///    * New host bootstrapped
///    * The module made a change to the system for itself to work (database initialized, user created)
#[macro_export(local_inner_macros)]
macro_rules! notice {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::log::Level::Notice, $($arg)+)
    );
    ($($arg:tt)+) => (
        log!($crate::log::Level::Notice, $($arg)+)
    )
}

/// Changes made to the system (usually 1 per repaired promise, more if the promise made multiple different changes to the system)
#[macro_export(local_inner_macros)]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::log::Level::Info, $($arg)+)
    );
    ($($arg:tt)+) => (
        log!($crate::log::Level::Info, $($arg)+)
    )
}

/// Human understandable detailed information about promise evaluation
#[macro_export(local_inner_macros)]
macro_rules! verbose {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::log::Level::Verbose, $($arg)+)
    );
    ($($arg:tt)+) => (
        log!($crate::log::Level::Verbose, $($arg)+)
    )
}

/// Programmer-level information that is only useful for CFEngine developers or module developers
#[macro_export(local_inner_macros)]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => (
        log!(target: $target, $crate::log::Level::Debug, $($arg)+)
    );
    ($($arg:tt)+) => (
        log!($crate::log::Level::Debug, $($arg)+)
    )
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_format_args {
    ($($args:tt)*) => {
        format_args!($($args)*)
    };
}
