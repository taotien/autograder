use std::str::{from_utf8, Utf8Error};

use miette::{Diagnostic, Report, SourceSpan};
use serde::Deserialize;
use similar::{DiffOp, TextDiff};
use thiserror::Error;
use tokio::process::Command;

use crate::config::Config;

// use crate::build::BuildSystem;

#[derive(Deserialize, Debug)]
pub struct Units {
    pub tests: Vec<Unit>,
}

#[derive(Deserialize, Debug)]
pub struct Unit {
    name: String,
    input: Vec<String>,
    expected: String,
    rubric: u64,
}

impl Unit {
    // Interpolate strings with '$' place holder
    pub fn interp_input(&mut self, config: &Config, executable: &str) {
        const PROJECT_DIR_SUBSTRING: &str = "$project";
        const DIGITAL_JAR_SUBSTRING: &str = "$digital";
        self.input
            .iter_mut()
            .map(|slice| {
                if slice.contains(PROJECT_DIR_SUBSTRING) {
                    *slice = slice.replace(PROJECT_DIR_SUBSTRING, executable)
                } else if slice.contains(DIGITAL_JAR_SUBSTRING) {
                    *slice = config.test.clone().unwrap().digital_path().to_string();
                }
            })
            .for_each(drop); // Consume the iterator
    }
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
#[diagnostic()]
pub struct IncorrectOutput {
    #[source_code]
    src: String,
    #[related]
    span_list: Vec<IncorrectSpan>,
}

#[derive(Error, Diagnostic, Debug, Clone)]
#[error("Want: {expected:?}, got: ")]
struct IncorrectSpan {
    expected: Option<String>,
    #[label("here")]
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
    pub async fn run(self) -> miette::Result<u64> {
        let mut tasks = Vec::with_capacity(self.tests.len());
        for unit in self.tests {
            tasks.push(tokio::spawn(unit.run()))
        }

        let mut outputs = Vec::with_capacity(tasks.len());
        for task in tasks {
            outputs.push(task.await.unwrap());
        }

        let grade: u64 = outputs
            .into_iter()
            .map(|out| {
                match out {
                    Ok(out) => out.grade, // Err(e) => bail!(e)
                    Err(e) => {
                        let report = Report::new(e);
                        eprintln!("{:?}", report);
                        0
                    }
                }
            })
            .sum();

        Ok(grade)
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

        let mut errors = vec![];
        let diff = TextDiff::from_lines(self.expected.as_ref(), stdout);
        let mut total_new_index = 0;
        let mut total_old_index = 0;
        for op in diff.ops() {
            // op.old_range()
            // op.new_range().count();
            // op.old_range().count();
            match op {
                DiffOp::Insert {
                    new_index, new_len, ..
                } => {
                    errors.push(IncorrectSpan {
                        expected: None,
                        at: (*new_index..*new_len).into(),
                    });
                }
                DiffOp::Delete {
                    old_index,
                    old_len,
                    new_index,
                } => errors.push(IncorrectSpan {
                    expected: self
                        .expected
                        .get(*old_index..*old_len)
                        .map(|s| s.to_owned()),
                    at: (*new_index..0).into(),
                }),
                DiffOp::Replace {
                    old_index,
                    old_len,
                    new_index,
                    new_len,
                } => errors.push(IncorrectSpan {
                    expected: self
                        .expected
                        .get(*old_index..*old_len)
                        .map(|s| s.to_owned()),
                    at: (*new_index..*new_len).into(),
                }),
                DiffOp::Equal { .. } => continue,
            }
        }

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

#[tokio::test]
async fn test_unit_run() -> miette::Result<()> {
    use miette::IntoDiagnostic;

    let test = Unit {
        name: "".into(),
        input: vec!["echo", "hello world"]
            .iter_mut()
            .map(|s| s.to_owned())
            .collect(),
        expected: "hello world\n".into(),
        rubric: 100,
    };
    test.run().await.into_diagnostic().unwrap();

    let test = Unit {
        name: "".into(),
        input: vec!["echo", "howdy y'all"]
            .iter_mut()
            .map(|s| s.to_owned())
            .collect(),
        expected: "hello world\n".into(),
        rubric: 100,
    };
    // test.run().await?;
    assert!(test.run().await.is_err());

    Ok(())
}
