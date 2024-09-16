use std::{process::ExitStatus, str::from_utf8};

use anyhow::Context;
use serde::Deserialize;
use tokio::process::Command;

#[derive(Deserialize, Debug)]
pub struct Tests {
    tests: Vec<Test>,
}

#[derive(Deserialize, Debug)]
struct Test {
    name: String,
    input: Vec<String>,
    expected: String,
    rubric: u64,
}

// fn pull_tests() {}

impl Tests {
    pub async fn run(self) -> anyhow::Result<u64> {
        let mut tasks = Vec::with_capacity(self.tests.len());

        // TODO or just

        for test in self.tests {
            tasks.push(tokio::spawn(test.run()))
        }

        let mut outputs = Vec::with_capacity(tasks.len());
        for task in tasks {
            outputs.push(task.await?);
        }

        let grade: u64 = outputs.iter().map(|o| o.as_ref().unwrap_or(&0)).sum();

        Ok(grade)
    }
}

impl Test {
    async fn run(self) -> anyhow::Result<u64> {
        let output = Command::new(&self.name)
            .args(&self.input)
            .output()
            .await
            .with_context(|| {
                format!(
                    "Could not spawn a child proccess, or get its output!
                    Tried to call: {}, with args {:?}",
                    self.name, self.input
                )
            })?;

        if !output.status.success() {}

        let stdout = from_utf8(&output.stdout).with_context(|| {
            format!(
                "Output does not contain valid utf8!
                Tried to call: {}, with args {:?}",
                self.name, self.input
            )
        })?;

        let grade;
        if self.expected == stdout {
            grade = self.rubric;
        } else {
            grade = 0;
        }

        Ok(grade)
    }
}
