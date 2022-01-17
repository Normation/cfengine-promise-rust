// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2021 Normation SAS

use crate::{
    attribute::AttributeType,
    header::Header,
    log::set_max_level,
    protocol::{
        EvaluateOutcome, EvaluateRequest, EvaluateResponse, ProtocolResult, TerminateRequest,
        TerminateResponse, ValidateRequest, ValidateResponse,
    },
    PromiseType,
};
use anyhow::{anyhow, Error};
use serde::Serialize;
use serde_json::{Map, Value};
use std::{
    io,
    io::{BufRead, Lines, Write},
    str::FromStr,
};

/// Promise executor
///
/// Handles communication with the CFEngine agent using the custom promise
/// JSON protocol on stdin/stdout.
#[derive(Default)]
pub struct Executor {
    /// Part of the executor as it is not a decision that belongs to the promise itself
    ignore_unknown_attributes: bool,
}

impl Executor {
    /// Create an executor
    ///
    /// By default it will fail on unexpected attributes
    pub fn new() -> Self {
        Self {
            ignore_unknown_attributes: false,
        }
    }

    /// Change behavior with unknown attributes
    pub fn ignore_unknown_attributes(mut self, ignore_unknown_attributes: bool) -> Self {
        self.ignore_unknown_attributes = ignore_unknown_attributes;
        self
    }

    /// Returns the output that would have been sent given provided input
    ///
    /// Useful for testing
    pub fn run_with_input<T: PromiseType>(
        &self,
        promise_type: T,
        input: &str,
    ) -> Result<String, Error> {
        let mut output = Vec::new();
        let mut error = Vec::new();

        self.run_type(promise_type, input.as_bytes(), &mut output, &mut error)?;

        let output = std::str::from_utf8(&output)?.to_string();
        Ok(output)
    }

    /// Runs a promise type for the agent, using stdio
    pub fn run<T: PromiseType>(&self, promise_type: T) -> Result<(), Error> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let stderr = io::stderr();

        let input = stdin.lock();
        let output = stdout.lock();
        let error = stderr.lock();

        self.run_type(promise_type, input, output, error)
    }

    /// Read a line followed by two empty lines
    fn read_line<B: BufRead>(input: &mut Lines<B>) -> Result<String, Error> {
        let line = input.next().unwrap()?;

        // Read exactly two empty lines
        for _n in 0..1 {
            let empty = input.next().unwrap()?;
            if !empty.is_empty() {
                return Err(anyhow!("Expecting two empty lines"));
            }
        }
        Ok(line)
    }

    /// Write lines followed by two empty lines
    fn write_line<W: Write>(output: &mut W, line: &str) -> Result<(), Error> {
        output.write_all(line.as_bytes())?;
        output.write_all(b"\n\n")?;
        output.flush()?;
        Ok(())
    }

    /// Write lines followed by two empty lines
    fn write_json<W: Write, L: Write, D: Serialize>(
        output: &mut W,
        _error: &mut L,
        data: D,
    ) -> Result<(), Error> {
        let json = serde_json::to_string(&data)?;
        Self::write_line(output, &json)
    }

    fn check_attributes(
        &self,
        attributes: &Map<String, Value>,
        required: Vec<(String, AttributeType)>,
        optional: Vec<(String, AttributeType)>,
    ) -> Result<(), Error> {
        for (attr, _) in &required {
            if attributes.get(attr).is_none() {
                anyhow!("Missing required attribute {}", attr);
            }
        }
        for (attr, attr_type) in required.iter().chain(optional.iter()) {
            if let Some(value) = attributes.get(attr) {
                if !attr_type.has_type(value) {
                    anyhow!("Attribute {} should have {:?} type", attr, attr_type);
                }
            }
        }
        if !self.ignore_unknown_attributes {
            for (key, _) in attributes {
                if required
                    .iter()
                    .chain(optional.iter())
                    .map(|(a, _)| a)
                    .all(|a| a != key)
                {
                    anyhow!("Unexpected attribute {}", key);
                }
            }
        }

        Ok(())
    }

    fn run_type<T: PromiseType, R: BufRead, W: Write, L: Write>(
        &self,
        mut promise: T,
        input: R,
        mut output: W,
        mut logger: L,
    ) -> Result<(), Error> {
        // Parse agent header
        let mut input = input.lines();
        let first_line = Self::read_line(&mut input)?;
        let header = Header::from_str(&first_line)?;
        header.compatibility()?;

        // Send my header
        let my_header =
            Header::new(promise.name().to_string(), promise.version().to_string()).to_string();
        Self::write_line(&mut output, &my_header)?;

        let mut initialized = false;

        // Now we're all set up, let's run the executor main loop
        loop {
            let line = Self::read_line(&mut input)?;
            let line = dbg!(line);
            // Lazily run initializer, in case it is expensive
            if !initialized {
                match promise.init() {
                    ProtocolResult::Failure(e) => {
                        return Err(anyhow!("failed to initialize promise type: {}", e));
                    }
                    ProtocolResult::Error(e) => {
                        return Err(anyhow!(
                            "failed to initialize promise type with unexpected: {}",
                            e
                        ));
                    }
                    ProtocolResult::Success => (),
                }
                initialized = true;
            }

            // Handle requests
            if let Ok(req) = serde_json::from_str::<ValidateRequest>(&line) {
                set_max_level(req.log_level);
                // Check parameters
                self.check_attributes(
                    &req.attributes,
                    promise.required_attributes(),
                    promise.optional_attributes(),
                )?;
                let result = promise.validate(&req.promiser, &req.attributes).outcome();
                Self::write_json(
                    &mut output,
                    &mut logger,
                    ValidateResponse::new(&req, result),
                )?
            } else if let Ok(req) = serde_json::from_str::<EvaluateRequest>(&line) {
                set_max_level(req.log_level);
                // FIXME fix once implemented
                let is_check_only = req.attributes.get("action_policy").is_some();

                let mut result = promise
                    .check(&req.promiser, &req.attributes)
                    .outcome(is_check_only);
                if !is_check_only && result != EvaluateOutcome::Kept {
                    // Make changes
                    result = promise.apply(&req.promiser, &req.attributes).outcome();
                }
                Self::write_json(
                    &mut output,
                    &mut logger,
                    EvaluateResponse::new(&req, result, vec![]),
                )?
            } else if let Ok(_req) = serde_json::from_str::<TerminateRequest>(&line) {
                let result = promise.terminate().outcome();
                Self::write_json(&mut output, &mut logger, TerminateResponse::new(result))?;
                return Ok(());
            } else {
                // Stop the program?
                return Err(anyhow!("Could not parse request: {}", line));
            };
        }
    }
}
