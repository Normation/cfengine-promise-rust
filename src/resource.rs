// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2021 Normation SAS

use serde::{Deserialize, Serialize};

// TODO model Rudder resource

// TODO abstract way cfengine stuff

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum ResourceKind {
    // We can check and apply
    State,
    // We're only able to check
    Check,
    // We always apply
    Action,
}
