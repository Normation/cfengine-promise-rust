// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2021 Normation SAS

use std::{fs, path::Path};

use rudder_resource::{
    name, version, ApplyResult, AttributeType, CheckResult, Executor, PromiseType,
};
use serde_json::{Map, Value};

struct Directory {}

impl PromiseType for Directory {
    name!("directory_module");
    version!("0.0.1");

    fn required_attributes(&self) -> Vec<(String, AttributeType)> {
        vec![(
            "state".to_string(),
            AttributeType::StringEnum(vec!["present".to_string(), "absent".to_string()]),
        )]
    }

    fn check(&mut self, promiser: &str, attributes: &Map<String, Value>) -> CheckResult {
        let should_be_present = attributes.get("state").unwrap().as_str().unwrap() == "present";

        match (should_be_present, Path::new(&promiser).exists()) {
            (true, true) | (false, false) => CheckResult::Kept,
            (true, false) => CheckResult::NotKept(format!(
                "Directory {} should be present but is not",
                promiser
            )),
            (false, true) => CheckResult::NotKept(format!(
                "Directory {} should not be present but is there",
                promiser
            )),
        }
    }

    fn apply(&mut self, promiser: &str, attributes: &Map<String, Value>) -> ApplyResult {
        let directory = Path::new(&promiser);
        let should_be_present = attributes.get("state").unwrap().as_str().unwrap() == "present";

        match (should_be_present, directory.exists()) {
            (true, true) | (false, false) => ApplyResult::Kept,
            (true, false) => match fs::create_dir(directory) {
                Ok(_) => {
                    ApplyResult::Repaired(format!("Created directory {}", directory.display()))
                }
                Err(e) => ApplyResult::NotKept(e.to_string()),
            },
            (false, true) => match fs::remove_dir(directory) {
                Ok(_) => {
                    ApplyResult::Repaired(format!("Removed directory {}", directory.display()))
                }
                Err(e) => ApplyResult::NotKept(e.to_string()),
            },
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let directory_promise_type = Directory {};
    // Run the promise executor
    Executor::new().run(directory_promise_type)
}
