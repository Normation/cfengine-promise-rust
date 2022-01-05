// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2021 Normation SAS

use cfengine_promise::{info, ApplyResult, AttributeType, CheckResult, Executor, PromiseType};
use serde_json::{Map, Value};
use std::{path::Path, process::Command};

struct Git {}

/// Implement the promise type
impl PromiseType for Git {
    fn name(&self) -> &'static str {
        "git_promise_module"
    }

    fn version(&self) -> &'static str {
        "0.0.1"
    }

    fn required_attributes(&self) -> Vec<(String, AttributeType)> {
        vec![("repo".to_string(), AttributeType::AbsolutePath)]
    }

    fn check(&mut self, promiser: &str, _attributes: &Map<String, Value>) -> CheckResult {
        if Path::new(&promiser).exists() {
            CheckResult::Kept
        } else {
            CheckResult::NotKept(format!("repo {} does not exist", promiser))
        }
    }

    fn apply(&mut self, promiser: &str, attributes: &Map<String, Value>) -> ApplyResult {
        let folder = Path::new(&promiser);
        // we have checked validity
        let url = attributes.get("repo").unwrap().as_str().unwrap();

        if folder.exists() {
            return ApplyResult::Kept;
        }

        info!("Cloning '{}' -> '{}'...", url, folder.display());

        match Command::new("git").args(&["clone", url, promiser]).output() {
            Err(e) => ApplyResult::NotKept(e.to_string()),
            Ok(_) => {
                if folder.exists() {
                    ApplyResult::Repaired(format!(
                        "Successfully cloned '{}' -> '{:?}'",
                        url, folder
                    ))
                } else {
                    ApplyResult::NotKept(
                        "git ran successfully but repo was not created".to_string(),
                    )
                }
            }
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let git_promise_type = Git {};
    // Run the promise executor
    Executor::new().run(git_promise_type)
}
