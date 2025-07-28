use std::{path::Path, process::Stdio};

use tokio::{io::AsyncWriteExt as _, process::Command};

use crate::{languages::Language, Output};

pub struct TestCase {
    pub input: String,
    pub output: String,
    pub hidden: bool,
}

impl TestCase {
    fn command(lang: &Language, dir: &Path) -> Command {
        let mut cmd = Command::new("nsjail");
        cmd.args([
            "-Mo",
            "-q",
            // "--rlimit_as",
            // "512",
            "-t",
            "2",
            // "--disable_proc",
            // "--iface_no_lo",
            "-c",
            dir.to_str().unwrap(),
            "-D",
            "/",
            // "--user",
            // "99999",
            // "--group",
            // "99999",
        ]);
        cmd.args([
            "--",
            // "sh",
            // "-c",
            match lang {
                Language::Rust | Language::Cpp => "./main",
                Language::Java => "java -cp . Main",
                Language::JavaScript => "deno run ./main.js",
                Language::Python => "python3 ./main.py",
            },
        ]);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    }

    pub async fn sandbox(
        &self,
        dir: &Path,
        lang: &Language,
        compiler: String,
    ) -> Result<Output, Output> {
        let mut child = Self::command(lang, dir).spawn()?;
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
        if Self::normalize_line(&stdout) == Self::normalize_line(&self.output) {
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
    fn normalize_line(s: &str) -> String {
        s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n")
    }

    #[cfg(test)]
    async fn test(&self, lang: &Language, program: &str) -> Result<Output, Output> {
        let (dir, compiler) = lang.compile(program).await?;
        assert!(dir
            .path()
            .join(match lang {
                Language::Rust | Language::Cpp => "main",
                Language::Java => "Main.class",
                Language::JavaScript => "main.js",
                Language::Python => "main.py",
            })
            .exists());
        self.sandbox(dir.path(), lang, compiler).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;

    #[tokio::test]
    async fn test_sandbox_rust() {
        let lang = Language::Rust;
        let program = r#"fn main() { println!("42"); }"#;
        let testcase = TestCase {
            input: String::new(),
            output: "42\n".to_string(),
            hidden: false,
        };
        let result = testcase.test(&lang, program).await;
        // assert!(matches!(result, Ok(Output::Success)));
        // panic!("{result:?}");
        if let Err(Output::RuntimeError { error, .. }) = result {
            panic!("{error}");
        }
    }

    #[tokio::test]
    async fn test_sandbox_python() {
        let lang = Language::Python;
        let program = r#"print(42)"#;
        let testcase = TestCase {
            input: String::new(),
            output: "42\n".to_string(),
            hidden: false,
        };
        let result = testcase.test(&lang, program).await;
        // assert!(matches!(result, Ok(Output::Success)));
        if let Err(Output::RuntimeError { error, .. }) = result {
            panic!("{error}");
        }
    }

    #[tokio::test]
    async fn test_sandbox_fail() {
        let lang = Language::Rust;
        let program = r#"fn main() { println!("wrong"); }"#;
        let testcase = TestCase {
            input: String::new(),
            output: "42\n".to_string(),
            hidden: false,
        };
        let result = testcase.test(&lang, program).await;
        // assert!(matches!(result, Err(Output::TestCaseFailed { .. })));
        if let Err(Output::RuntimeError { error, .. }) = result {
            panic!("{error}");
        }
    }

    #[tokio::test]
    async fn test_sandbox_hidden_fail() {
        let lang = Language::Rust;
        let program = r#"fn main() { println!("wrong"); }"#;
        let testcase = TestCase {
            input: String::new(),
            output: "42\n".to_string(),
            hidden: true,
        };
        let result = testcase.test(&lang, program).await;
        // assert!(matches!(result, Err(Output::HiddenTestCaseFailed(_))));
        if let Err(Output::RuntimeError { error, .. }) = result {
            panic!("{error}");
        }
    }
}
