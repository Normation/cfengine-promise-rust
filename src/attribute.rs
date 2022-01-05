// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2021 Normation SAS

use serde_json::Value;
use std::path::Path;

/// First-level type of attributes
///
/// Allows providing typing information
#[derive(Debug, PartialEq, Clone)]
pub enum AttributeType {
    Bool,
    String,
    Integer,
    Float,
    List,
    Data,
    /// Can only be one of the given variant
    StringEnum(Vec<String>),
    AbsolutePath,
    // TODO extend with usual types for config management
}

impl AttributeType {
    pub(crate) fn has_type(&self, value: &Value) -> bool {
        match self {
            AttributeType::Bool => value.as_bool().is_some(),
            AttributeType::String => value.as_str().is_some(),
            AttributeType::Integer => value.as_i64().is_some(),
            AttributeType::Float => value.as_f64().is_some(),
            AttributeType::List => value.as_array().is_some(),
            AttributeType::Data => value.as_object().is_some(),
            AttributeType::AbsolutePath => value
                .as_str()
                .map(|p| Path::new(p).is_absolute())
                .unwrap_or(false),
            AttributeType::StringEnum(e) => value
                .as_str()
                .map(|s| e.contains(&s.to_owned()))
                .unwrap_or(false),
        }
    }
}
