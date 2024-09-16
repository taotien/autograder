use std::{process::ExitStatus, str::from_utf8};

use anyhow::{bail, Context};
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
        let make_output = Command::new("make").output().await.with_context(|| {
            format!(
                "Could not spawn a child proccess, or get its output!
                    Tried to call: make",
            )
        })?;

        if !make_output.status.success() {
            println!("make failed!");
            let make_stdout = from_utf8(&make_output.stdout)?;
            println!("{}", make_stdout);

            return Ok(0);
        }

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

        // TODO handle stderr
        // if !output.status.success() {
        //     println!("command fiald!");
        //     println!("{}", from_utf8(&output.stdout)?);
        // }

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
            println!("expected: {}", self.expected);
            println!("got '{}'", stdout);
            grade = 0;
        }

        Ok(grade)
    }
}
