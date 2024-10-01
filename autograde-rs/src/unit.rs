use std::{process::Output, str::from_utf8};

use anyhow::Context;
use miette::{Diagnostic, SourceSpan};
use serde::Deserialize;
use similar::TextDiff;
use thiserror::Error;
use tokio::process::Command;

// use crate::build::BuildSystem;

#[derive(Deserialize, Debug)]
pub struct Units {
    tests: Vec<Unit>,
}

#[derive(Deserialize, Debug)]
struct Unit {
    name: String,
    input: Vec<String>,
    expected: String,
    rubric: u64,
}

#[derive(Debug)]
struct UnitOutput {
    // output: String,
    grade: u64,
}

#[derive(Error, Diagnostic, Debug)]
#[error("One or more tests failed")]
pub struct UnitErrors {
    #[related]
    errors: Vec<UnitError>,
}

#[derive(Error, Diagnostic, Debug)]
pub enum UnitError {
    // #[error("Exit code wasn't zero")]
    // NonZeroExit,
    #[error("Program crashed")]
    ProgramCrashed,
    #[error(transparent)]
    #[diagnostic(transparent)]
    IncorrectOutput(#[from] IncorrectOutput),
    #[error("Could not run program")]
    Wrapped(std::io::Error),
}

// #[derive(Error, Diagnostic, Debug)]
// #[error("Not all tests passed")]
// pub struct UnitErrors {
//     #[source_code]
//     src: String,
//     #[related]
//     related: Vec<UnitError>,
// }

#[derive(Error, Diagnostic, Debug)]
#[error("Output doesn't match expected result")]
// #[diagnostic(
//     help("")
// )]
pub struct IncorrectOutput {
    #[source_code]
    src: String,
    // #[label("here")]
    #[related]
    span_list: Vec<IncorrectSpan>,
}

#[derive(Error, Diagnostic, Debug, Clone)]
#[error("Output doesn't match expected result")]
struct IncorrectSpan {
    at: SourceSpan,
}

// fn pull_tests() {}

// #[allow(async_fn_in_trait)]
// pub trait RunProject {
//     async fn run(self) -> miette::Result<u64>;
// }

// #[allow(async_fn_in_trait)]
// pub trait RunUnit {
//     async fn run(&self) -> Result<TestOutput, UnitError>;
// }

// impl RunProject for Units {
impl Units {
    pub async fn run(self) -> Result<u64, UnitErrors> {
        let mut tasks = Vec::with_capacity(self.tests.len());
        for unit in self.tests {
            tasks.push(tokio::spawn(unit.run()))
        }

        let mut outputs = Vec::with_capacity(tasks.len());
        for task in tasks {
            outputs.push(task.await.unwrap());
        }

        let mut errors = vec![];
        let grade: u64 = outputs
            .into_iter()
            .map(|out| {
                match out {
                    Ok(out) => out.grade, // Err(e) => bail!(e)
                    Err(e) => {
                        errors.push(e);
                        0
                    }
                }
            })
            .sum();

        // let errors: Vec<_> = errors
        //     .into_iter()
        //     // .map(|e| e.to_owned())
        //     .flatten()
        //     // .map(|e| e.clone())
        //     .collect();

        if errors.is_empty() {
            Ok(grade)
        } else {
            Err(UnitErrors { errors })
        }
    }
}

// impl RunUnit for Unit {
impl Unit {
    async fn run(self) -> Result<UnitOutput, UnitError> {
        let output = Command::new(&self.name)
            .args(&self.input)
            .output()
            .await
            .map_err(|e| UnitError::Wrapped(e))?;

        // TODO do we care about nonzero exits?
        // if !output.status.success() {
        // }

        let stdout = from_utf8(&output.stdout)
            .with_context(|| {
                format!(
                    "Output does not contain valid utf8!
                    Tried to call: {}, with args {:?}",
                    self.name, self.input
                )
            })
            .unwrap();

        let diff = TextDiff::from_lines(self.expected.as_ref(), stdout);

        let mut errors = vec![];
        for op in diff.ops() {
            let range = op.new_range();
            let err = IncorrectSpan {
                // src: stdout.into(),
                at: range.into(),
            };
            errors.push(err);
        }

        if errors.is_empty() {
            Ok(UnitOutput {
                // output: stdout.into(),
                grade: self.rubric,
            })
        } else {
            Err(UnitError::IncorrectOutput(IncorrectOutput {
                src: stdout.into(),
                span_list: errors,
            }))
        }
    }
}
