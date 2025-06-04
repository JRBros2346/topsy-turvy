use std::{io, path::Path, process::Stdio};

use ::futures::future;
use serde::Deserialize;
use tempfile::TempDir;
use tokio::{fs::File, io::AsyncWriteExt, process::Command};

async fn submit(submission: Submission, test_cases: Vec<TestCase>) -> Result<Output, Output> {
    let (dir, compile_error) = submission.program.0.compile(&submission.program.1).await?;
    let futures = test_cases.into_iter().map(|tc| {
        let dir = dir.path().to_path_buf();
        let lang = submission.program.0.clone();
        let compiler = compile_error.clone();
        tokio::spawn(async move { tc.run(&dir, &lang, compiler).await })
    });
    let results = future::join_all(futures).await;
    for res in results {
        match res {
            Ok(Err(e)) => return Err(e),
            Ok(Ok(_)) => continue,
            Err(e) => return Err(Output::IO(io::Error::new(io::ErrorKind::Other, e))),
        }
    }
    Ok(Output::Success)
}

#[derive(Deserialize)]
pub struct Submission {
    pub problem: usize,
    pub program: (Language, String),
}

pub struct TestCase {
    pub input: String,
    pub output: String,
    pub hidden: bool,
}

#[non_exhaustive]
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    JavaScript,
    Cpp,
    Python,
    Java,
}

impl Language {
    async fn compile(&self, program: &str) -> Result<(TempDir, String), Output> {
        use Language::*;
        let dir = TempDir::new()?;
        let file = dir.path().join(match self {
            Rust => "main.rs",
            JavaScript => "main.js",
            Cpp => "main.cpp",
            Python => "main.py",
            Java => "Main.java",
        });
        let mut f = File::create(&file).await?;
        f.write_all(program.as_bytes()).await?;

        match self {
            JavaScript | Python => return Ok((dir, String::new())),
            lang => {
                let mut output = match lang {
                    Rust => Command::new("rustc"),
                    Cpp => Command::new("clang++"),
                    Java => Command::new("javac"),
                    _ => unreachable!(),
                };
                output.arg(file);
                match lang {
                    Rust => output.arg("--color=always"),
                    Cpp => output.arg("-o").arg("main").arg("-fcolor-diagnostics"),
                    Java => &mut output,
                    _ => unreachable!(),
                };
                let output = output.current_dir(dir.path()).output().await?;
                let error = String::from_utf8_lossy(&output.stderr).to_string();
                if !output.status.success() {
                    return Err(Output::CompileError(error));
                }
                Ok((dir, error))
            }
        }
    }
}

impl TestCase {
    async fn run(&self, dir: &Path, lang: &Language, compiler: String) -> Result<Output, Output> {
        use Language::*;
        let mut child = Command::new("nsjail")
            .args([
                "--mode",
                "o",
                "--quiet",
                "--rlimit_as",
                "512",
                "--time_limit",
                "2",
                "--disable_proc",
                "--iface_no_lo",
                "--chroot",
                dir.to_str().unwrap(),
                "--user",
                "99999",
                "--group",
                "99999",
                "sh",
                "-c",
                match lang {
                    Rust | Cpp => "./main",
                    Java => "java Main",
                    JavaScript => "deno run main.js",
                    Python => "python3 main.py",
                },
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(self.input.as_bytes()).await?;
        }
        let output = child.wait_with_output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            return Err(Output::RuntimeError {
                compiler,
                input: self.input.clone(),
                output: stdout,
                error: stderr,
            });
        }
        if stdout
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
            == self
                .output
                .lines()
                .map(str::trim_end)
                .collect::<Vec<_>>()
                .join("\n")
        {
            Ok(Output::Success)
        } else if self.hidden {
            Err(Output::HiddenTestCaseFailed(compiler))
        } else {
            Err(Output::TestCaseFailed {
                compiler,
                input: self.input.clone(),
                error: stderr,
                expected: self.output.clone(),
                actual: stdout,
            })
        }
    }
}

#[derive(Debug)]
pub enum Output {
    IO(io::Error),
    CompileError(String),
    RuntimeError {
        compiler: String,
        input: String,
        output: String,
        error: String,
    },
    TestCaseFailed {
        compiler: String,
        input: String,
        error: String,
        expected: String,
        actual: String,
    },
    HiddenTestCaseFailed(String),
    Success,
}
impl From<io::Error> for Output {
    fn from(err: io::Error) -> Self {
        Output::IO(err)
    }
}
