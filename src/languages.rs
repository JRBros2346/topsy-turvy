use std::path::Path;

use serde::Deserialize;
use tempfile::TempDir;
use tokio::{fs::File, io::AsyncWriteExt as _, process::Command};

use crate::Output;

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
    pub async fn compile(&self, program: &str) -> Result<(TempDir, String), Output> {
        let dir = TempDir::new()?;
        let file = dir.path().join(self.file_name());
        let mut f = File::create(&file).await?;
        f.write_all(program.as_bytes()).await?;

        if self.is_interpreted() {
            return Ok((dir, String::new()));
        }

        let output = self.compile_command(&file, dir.path()).output().await?;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            return Err(Output::CompileError(stderr));
        }
        Ok((dir, stderr))
    }

    fn file_name(&self) -> &'static str {
        match self {
            Language::Rust => "main.rs",
            Language::JavaScript => "main.js",
            Language::Cpp => "main.cpp",
            Language::Python => "main.py",
            Language::Java => "Main.java",
        }
    }
    fn is_interpreted(&self) -> bool {
        matches!(self, Language::JavaScript | Language::Python)
    }
    fn compile_command(&self, file: &Path, dir: &Path) -> Command {
        let mut cmd = match self {
            Language::Rust => Command::new("rustc"),
            Language::Cpp => Command::new("clang++"),
            Language::Java => Command::new("javac"),
            _ => unreachable!(),
        };
        match self {
            Language::Rust => cmd.args(["--color=always", "--target", "x86_64-unknown-linux-musl"]),
            Language::Cpp => cmd.args(["-o", "main", "-fcolor-diagnostics", "-static"]),
            Language::Java => &mut cmd,
            _ => unreachable!(),
        };
        cmd.arg(file).current_dir(dir);
        cmd
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_compile_rust() {
        use super::*;
        let lang = Language::Rust;
        let program = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;
        let (dir, compile_error) = lang.compile(program).await.unwrap();
        assert!(compile_error.is_empty());
        assert!(dir.path().join("main").exists());
    }

    #[tokio::test]
    async fn test_compile_rust_with_error() {
        use super::*;
        let lang = Language::Rust;
        let program = "";
        match lang.compile(program).await.unwrap_err() {
            Output::CompileError(compile_error) => assert!(!compile_error.is_empty()),
            _ => unreachable!(),
        }
    }

    #[tokio::test]
    async fn test_compile_javascript() {
        use super::*;
        let lang = Language::JavaScript;
        let program = r#"
            console.log("Hello, world!");
        "#;
        let (dir, compile_error) = lang.compile(program).await.unwrap();
        assert!(compile_error.is_empty());
        assert!(dir.path().join("main.js").exists());
    }

    #[tokio::test]
    async fn test_compile_cpp() {
        use super::*;
        let lang = Language::Cpp;
        let program = r#"
            #include <iostream>
            int main() {
                std::cout << "Hello, world!" << std::endl;
                return 0;
            }
        "#;
        let (dir, compile_error) = lang.compile(program).await.unwrap();
        assert!(compile_error.is_empty());
        assert!(dir.path().join("main").exists());
    }

    #[tokio::test]
    async fn test_compile_python() {
        use super::*;
        let lang = Language::Python;
        let program = r#"
            print("Hello, world!")
        "#;
        let (dir, compile_error) = lang.compile(program).await.unwrap();
        assert!(compile_error.is_empty());
        assert!(dir.path().join("main.py").exists());
    }

    #[tokio::test]
    async fn test_compile_java() {
        use super::*;
        let lang = Language::Java;
        let program = r#"
            public class Main {
                public static void main(String[] args) {
                    System.out.println("Hello, world!");
                }
            }
        "#;
        let (dir, compile_error) = lang.compile(program).await.unwrap();
        assert!(compile_error.is_empty());
        assert!(dir.path().join("Main.class").exists());
    }
}
