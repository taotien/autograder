use std::str::{from_utf8, Utf8Error};

use anyhow::Context;
use miette::{Diagnostic, SourceSpan};
use serde::Deserialize;
use similar::{ChangeTag, DiffOp, TextDiff};
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
    #[source_code]
    src: String,
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
    #[error("Not UTF8")]
    NotUtf8(Utf8Error),
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
    #[label(collection, "here")]
    span_list: Vec<SourceSpan>,
}

// #[derive(Error, Diagnostic, Debug, Clone)]
// #[error("output")]
// struct IncorrectSpan {
//     #[label("here")]
//     at: SourceSpan,
// }

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
    pub async fn run(self) -> miette::Result<u64> {
        let mut tasks = Vec::with_capacity(self.tests.len());
        for unit in self.tests {
            tasks.push(tokio::spawn(unit.run()))
        }

        let mut outputs = Vec::with_capacity(tasks.len());
        for task in tasks {
            outputs.push(task.await.unwrap());
        }

        // let mut errors = vec![];
        // let grade: u64 = outputs
        //     .into_iter()
        //     .map(|out| {
        //         match out {
        //             Ok(out) => out.grade, // Err(e) => bail!(e)
        //             Err(e) => {
        //                 errors.push(e);
        //                 0
        //             }
        //         }
        //     })
        //     .sum();

        for out in outputs {
            out?;
        }

        // let errors: Vec<_> = errors
        //     .into_iter()
        //     // .map(|e| e.to_owned())
        //     .flatten()
        //     // .map(|e| e.clone())
        //     .collect();

        // if errors.is_empty() {
        todo!()
        // Ok(grade)
        // } else {
        //     // Err(UnitErrors { errors })
        // }
    }
}

// impl RunUnit for Unit {
impl Unit {
    async fn run(self) -> Result<UnitOutput, UnitError> {
        let output = Command::new(&self.input.first().expect("Empty input in tests file!"))
            .args(
                self.input
                    .split_first()
                    .expect("Empty input in tests file!")
                    .1,
            )
            .output()
            .await
            .map_err(UnitError::Wrapped)?;

        // TODO do we care about nonzero exits?
        // if !output.status.success() {
        // }

        let stdout = from_utf8(&output.stdout).map_err(|e| UnitError::NotUtf8(e))?;

        let diff = TextDiff::from_lines(self.expected.as_ref(), stdout);

        let mut errors = vec![];
        for op in diff.ops() {
            match op {
                DiffOp::Insert {
                    new_index, new_len, ..
                } => errors.push((*new_index..*new_len).into()),
                DiffOp::Delete { new_index, .. } => errors.push((*new_index..0).into()),
                DiffOp::Replace {
                    new_index, new_len, ..
                } => errors.push((*new_index..*new_len).into()),
                DiffOp::Equal { .. } => continue,
            }
            // let range = op.new_range();
            // errors.push(range.into());
        }

        // println!("{:?}", diff.iter_all_changes());
        // for change in diff.iter_all_changes() {
        //     let sign = match change.tag() {
        //         ChangeTag::Delete => "-",
        //         ChangeTag::Insert => "+",
        //         ChangeTag::Equal => " ",
        //     };
        //     print!("{}{}", sign, change);
        // }

        // println!("{}", stdout);

        if errors.is_empty() {
            Ok(UnitOutput { grade: self.rubric })
        } else {
            Err(UnitError::IncorrectOutput(IncorrectOutput {
                src: stdout.into(),
                span_list: errors,
            }))
        }
    }
}
