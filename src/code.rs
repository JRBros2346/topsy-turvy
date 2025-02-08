use crate::submit::TestCase;
use crate::{Language, Output};
use std::path::Path;
use tokio::process::Command;
use tokio::time::{self, Duration, Instant};

#[tracing::instrument(name = "compile_code", skip(code, dir))]
pub async fn compile_code(code: &str, language: Language, dir: &Path) -> Result<(), Output> {
    use std::fs;
    let file = match language {
        Language::Cpp => "main.cpp",
        Language::Rust => "main.rs",
        Language::Javascript => "main.js",
        Language::Python => "main.py",
        Language::Java => "Main.java",
    };
    fs::write(&dir.join(file), code)
        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
        .map_err(|_| Output::ServerError)?;
    if !language.is_compiled() {
        return Ok(());
    }
    let compiler = match language {
        Language::Cpp => "clang++",
        Language::Rust => "rustc",
        Language::Java => "javac",
        _ => unreachable!(),
    };
    let args: Box<dyn Iterator<Item = &str>> = match language {
        Language::Cpp => Box::new([file, "-o", "main"].into_iter()),
        Language::Rust | Language::Java => Box::new([file].into_iter()),
        _ => unreachable!(),
    };
    let output = Command::new(compiler)
        .current_dir(dir)
        .args(args)
        .output()
        .await
        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
        .map_err(|_| Output::ServerError)?;
    if !output.status.success() {
        return Err(Output::CannotCompile(
            String::from_utf8_lossy(&strip_ansi_escapes::strip(output.stderr)).to_string(),
        ));
    }
    Ok(())
}

#[tracing::instrument(name = "test_code", skip(test, dir))]
pub async fn test_code(language: Language, test: TestCase, dir: &Path) -> Result<Duration, Output> {
    use std::process::Stdio;
    let runtime = match language {
        Language::Cpp | Language::Rust => "./main",
        Language::Javascript => "deno",
        Language::Python => "python3",
        Language::Java => "java",
    };
    let args: Box<dyn Iterator<Item = &str>> = match language {
        Language::Cpp | Language::Rust => Box::new([].into_iter()),
        Language::Javascript => Box::new(["run", "main.js"].into_iter()),
        Language::Python => Box::new(["main.py"].into_iter()),
        Language::Java => Box::new(["Main"].into_iter()),
    };
    let mut child = Command::new(runtime)
        .current_dir(dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
        .map_err(|_| Output::ServerError)?;
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt as _;
        stdin
            .write_all(test.input.as_bytes())
            .await
            .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
            .map_err(|_| Output::ServerError)?;
    }
    let start = Instant::now();
    let execution = time::timeout(Duration::from_secs(5), child.wait_with_output()).await;
    let result = start.elapsed();
    let output = execution
        .map_err(|_| Output::Timeout(test.clone()))?
        .inspect_err(|e| tracing::error!("{e} {} {}", file!(), line!()))
        .map_err(|_| Output::ServerError)?;
    let stdout = String::from_utf8_lossy(&strip_ansi_escapes::strip(output.stdout)).to_string();
    let stderr = String::from_utf8_lossy(&strip_ansi_escapes::strip(output.stderr)).to_string();
    if !output.status.success() {
        return Err(Output::RuntimeError { stderr, stdout });
    }
    if stdout.trim() != test.output {
        return Err(Output::WrongAnswer {
            test,
            stdout,
            stderr,
        });
    }
    Ok(result)
}
