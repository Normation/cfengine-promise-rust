// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2021 Normation SAS

//! A Rust implementation of CFEngine's custom promise types
//!
//! The main goal is to provide a reliable interface, by checking as much elements as we can
//! (including parameters types, etc) to allow easily implementing safe and fast promise types.
//! Note that we do not try to stick to close to the underlying protocol, and prefer a
//! an idiomatic way.
//!
//! This lib is done with [Rudder](https://www.rudder.io) use cases in mind, so we have a special focus on the audit mode (warn only).
//! In this order, we split the *evaluate* step into *check* and *apply*
//! to handle warn-only at executor level and avoid having to implement it in every promise.
//!
//! The library is generally build around a `PromiseType` trait describing a promise type's interface, and an `Executor`
//! that handles the stdin/stdout communication and protocol serialization.

pub use crate::{
    attribute::AttributeType,
    executor::Executor,
    protocol::{ApplyResult, CheckResult, ProtocolResult, ValidateResult},
};
pub use serde_json::{Map, Value};

mod attribute;
mod executor;
mod header;
#[macro_use]
pub mod log;
mod protocol;

/// CFEngine promise type
pub trait PromiseType {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    // no protocol versions as it is part of the executor

    /// Executed before any promise
    ///
    /// Can be used for set-up tasks
    fn init(&mut self) -> ProtocolResult {
        ProtocolResult::Success
    }

    /// List of required attributes with their type
    ///
    /// They will be checked before calling `validate`
    ///
    /// Uses a `Vec` and `String`s to allow
    /// dynamic attribute list (depending on OS, etc.)
    fn required_attributes(&self) -> Vec<(String, AttributeType)> {
        vec![]
    }

    /// List of optional attributes with their type
    ///
    /// They will be checked before calling `validate`
    ///
    /// Uses a `Vec` and `String`s to allow
    /// dynamic attribute list (depending on OS, etc.)
    fn optional_attributes(&self) -> Vec<(String, AttributeType)> {
        vec![]
    }

    /// Checks parameter validity
    ///
    /// Should be used for parameters validation, additionally to
    /// `required_attributes` and `optional_attributes`.
    fn validate(&self, _promiser: &str, _attributes: &Map<String, Value>) -> ValidateResult {
        ValidateResult::Valid
    }

    /// Test if the policy is applied
    ///
    /// Assumes validation has already been done.
    ///
    /// Does not need to be implemented for promises that should be evaluated every time
    /// (usually actions).
    fn check(&mut self, _promiser: &str, _attributes: &Map<String, Value>) -> CheckResult {
        CheckResult::AlwaysApply
    }

    /// Apply the policy and make changes
    ///
    /// Assumes validation has already been done
    fn apply(&mut self, _promiser: &str, _attributes: &Map<String, Value>) -> ApplyResult {
        ApplyResult::AuditOnly
    }

    /// Run before normal executor termination,
    /// can be used for clean up tasks.
    fn terminate(&mut self) -> ProtocolResult {
        ProtocolResult::Success
    }
}
