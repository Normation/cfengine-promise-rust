// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2021 Normation SAS

use crate::{error, info, log::LevelFilter};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
/// Promise validation outcomes
pub(crate) enum ValidateOutcome {
    /// Validation successful
    Valid,
    /// Validation failed, error in cfengine policy
    Invalid,
    /// Unexpected error
    Error,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]

/// Promise validation result
pub enum ValidateResult {
    /// Validation successful
    Valid,
    /// Validation failed, error in cfengine policy
    ///
    /// Parameter will be logged at error level
    Invalid(String),
    /// Unexpected error
    ///
    /// Parameter will be logged at error level
    Error(String),
}

impl ValidateResult {
    pub(crate) fn outcome(&self) -> ValidateOutcome {
        match self {
            ValidateResult::Valid => ValidateOutcome::Valid,
            ValidateResult::Invalid(e) => {
                error!("{}", e);
                ValidateOutcome::Invalid
            }
            ValidateResult::Error(e) => {
                error!("{}", e);
                ValidateOutcome::Error
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
/// Promise evaluation outcomes
pub(crate) enum EvaluateOutcome {
    /// Satisfied already, no change
    Kept,
    /// Not satisfied before, but fixed
    Repaired,
    /// Not satisfied before, not fixed
    NotKept,
    /// Unexpected error
    Error,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]

/// Promise application result
pub enum ApplyResult {
    /// Satisfied already, no change
    Kept,
    /// Not satisfied before, but fixed
    ///
    /// Parameter will be logged at info level
    Repaired(String),
    /// Not satisfied before, not fixed
    ///
    /// Parameter will be logged at error level
    NotKept(String),
    /// Unexpected error
    ///
    /// Parameter will be logged at critical level
    Error(String),
    /// A promise that should never be applied but only checked
    AuditOnly,
}

impl ApplyResult {
    pub(crate) fn outcome(&self) -> EvaluateOutcome {
        match self {
            ApplyResult::Kept => EvaluateOutcome::Kept,
            ApplyResult::Repaired(m) => {
                info!("{}", m);
                EvaluateOutcome::Repaired
            }
            ApplyResult::NotKept(e) => {
                error!("{}", e);
                EvaluateOutcome::NotKept
            }
            ApplyResult::Error(e) => {
                error!("{}", e);
                EvaluateOutcome::Error
            }
            ApplyResult::AuditOnly => {
                error!("Should not be applied, audit only promise");
                EvaluateOutcome::Error
            }
        }
    }
}

/// Promise evaluation result
///
/// Used as audit result if in warn_only
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum CheckResult {
    /// Satisfied already, no change
    Kept,
    /// This promise should always be applied, as it is an action without
    /// sensible test method
    AlwaysApply,
    /// Not satisfied before, not fixed
    ///
    /// Parameter will be logged at error level
    NotKept(String),
    /// Unexpected error
    ///
    /// Parameter will be logged at critical level
    Error(String),
}

impl CheckResult {
    /// If is_check_only, not kept is logged as error, else as info
    pub(crate) fn outcome(&self, is_check_only: bool) -> EvaluateOutcome {
        match self {
            CheckResult::Kept => EvaluateOutcome::Kept,
            CheckResult::AlwaysApply => {
                info!("This promise needs to be applied");
                EvaluateOutcome::NotKept
            }
            CheckResult::NotKept(e) => {
                if is_check_only {
                    error!("{}", e);
                } else {
                    info!("{}", e);
                }
                EvaluateOutcome::NotKept
            }
            CheckResult::Error(e) => {
                error!("{}", e);
                EvaluateOutcome::Error
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
/// Result for init/terminate
pub(crate) enum ProtocolOutcome {
    /// Success
    Success,
    /// Error
    Failure,
    /// Unexpected error
    Error,
}

/// Init/Terminate result
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ProtocolResult {
    /// Success
    Success,
    /// Error
    ///
    /// Parameter will be logged at error level
    Failure(String),
    /// Unexpected error
    ///
    /// Parameter will be logged at error level
    Error(String),
}

impl ProtocolResult {
    pub(crate) fn outcome(&self) -> ProtocolOutcome {
        match self {
            ProtocolResult::Success => ProtocolOutcome::Success,
            ProtocolResult::Failure(e) => {
                error!("{}", e);
                ProtocolOutcome::Failure
            }
            ProtocolResult::Error(e) => {
                error!("{}", e);
                ProtocolOutcome::Error
            }
        }
    }
}

// Little hack for constant tags in serialized JSON
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum ValidateOperation {
    ValidatePromise,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum EvaluateOperation {
    EvaluatePromise,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum TerminateOperation {
    Terminate,
}

// {"operation": "validate_promise", "log_level": "info", "promise_type": "git", "promiser": "/opt/cfengine/masterfiles", "attributes": {"repo": "https://github.com/cfengine/masterfiles"}}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct ValidateRequest {
    operation: ValidateOperation,
    pub(crate) log_level: LevelFilter,
    pub(crate) promiser: String,
    pub(crate) attributes: Map<String, Value>,
}

// {"operation": "evaluate_promise", "log_level": "info", "promise_type": "git", "promiser": "/opt/cfengine/masterfiles", "attributes": {"repo": "https://github.com/cfengine/masterfiles"}}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct EvaluateRequest {
    operation: EvaluateOperation,
    pub(crate) log_level: LevelFilter,
    pub(crate) promiser: String,
    pub(crate) attributes: Map<String, Value>,
}

// {"operation": "terminate", "log_level": "info"}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct TerminateRequest {
    operation: TerminateOperation,
}

////////////////////////////////////

// {"operation": "validate_promise", "promiser": "/opt/cfengine/masterfiles", "attributes": {"repo": "https://github.com/cfengine/masterfiles"}, "result": "valid"}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct ValidateResponse {
    operation: ValidateOperation,
    promiser: String,
    attributes: Map<String, Value>,
    result: ValidateOutcome,
}

impl ValidateResponse {
    pub(crate) fn new(request: &ValidateRequest, result: ValidateOutcome) -> Self {
        Self {
            operation: ValidateOperation::ValidatePromise,
            promiser: request.promiser.clone(),
            result,
            attributes: request.attributes.clone(),
        }
    }
}

// {"operation": "evaluate_promise", "promiser": "/opt/cfengine/masterfiles", "attributes": {"repo": "https://github.com/cfengine/masterfiles"}, "result": "kept"}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct EvaluateResponse {
    operation: EvaluateOperation,
    promiser: String,
    attributes: Map<String, Value>,
    result: EvaluateOutcome,
}

impl EvaluateResponse {
    pub(crate) fn new(request: &EvaluateRequest, result: EvaluateOutcome) -> Self {
        Self {
            operation: EvaluateOperation::EvaluatePromise,
            promiser: request.promiser.clone(),
            result,
            attributes: request.attributes.clone(),
        }
    }
}

// {"operation": "terminate", "result": "success"}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct TerminateResponse {
    operation: TerminateOperation,
    result: ProtocolOutcome,
}

impl TerminateResponse {
    pub(crate) fn new(result: ProtocolOutcome) -> Self {
        Self {
            operation: TerminateOperation::Terminate,
            result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_requests() {
        let val = r#"{"attributes":{"repo":"https://github.com/cfengine/masterfiles"},"log_level":"info","operation":"validate_promise","promiser":"/tmp/masterfiles"}"#;

        let mut attributes = Map::new();
        attributes.insert(
            "repo".to_string(),
            Value::String("https://github.com/cfengine/masterfiles".to_string()),
        );
        let ref_val = ValidateRequest {
            operation: ValidateOperation::ValidatePromise,
            log_level: LevelFilter::Info,
            promiser: "/tmp/masterfiles".to_string(),
            attributes,
        };

        assert_eq!(
            serde_json::from_str::<ValidateRequest>(&val).unwrap(),
            ref_val
        );
    }
}
