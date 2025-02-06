use std::path::Path;

use tokio::time::{self, Duration, Instant};
use tokio::process::Command;

use crate::submit::TestCase;
use crate::{Language, Output};

pub async fn compile_code(code: &str, language: Language, dir: &Path) -> Result<(), Output> {
    use std::fs;
    let file = match language {
        Language::Cpp => "main.cpp",
        Language::Rust => "main.rs",
        Language::Javascript => "main.js",
        Language::Python => "main.py",
        Language::Java => "Main.java",
    };
    if let Err(err) = fs::write(&dir.join(file), code) {
        eprintln!("{err} {} {}", file!(), line!());
        return Err(Output::ServerError);
    }
    if !language.is_compiled() {
        return Ok(());
    }
    let compiler = match language {
        Language::Cpp => "clang++",
        Language::Rust => "rustc",
        Language::Java => "javac",
        _ => unreachable!()
    };
    let args: Box<dyn Iterator<Item = &str>> = match language {
        Language::Cpp => Box::new([file, "-o", "main"].into_iter()),
        Language::Rust | Language::Java => Box::new([file].into_iter()),
        _ => unreachable!()
    };
    let output = match Command::new(compiler)
        .current_dir(dir)
        .args(args)
        .output()
        .await
    {
        Ok(output) => output,
        Err(err) => {
            eprintln!("{err} {} {}", file!(), line!());
            return Err(Output::ServerError);
        }
    };
    if !output.status.success() {
        return Err(Output::CannotCompile(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    Ok(())
}

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
    let mut child = match Command::new(runtime)
        .current_dir(dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            eprintln!("{err} {} {}", file!(), line!());
            return Err(Output::ServerError);
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt as _;
        if let Err(err) = stdin.write_all(test.input.as_bytes()).await {
            eprintln!("{err} {} {}", file!(), line!());
            return Err(Output::ServerError);
        }
    }
    let start = Instant::now();
    let execution = time::timeout(Duration::from_secs(5), child.wait_with_output()).await;
    let result = start.elapsed();
    match execution {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if !output.status.success() {
                return Err(Output::RuntimeError { stderr, stdout });
            }
            let out = String::from_utf8_lossy(&output.stdout).to_string();
            if out.trim() != test.output {
                return Err(Output::WrongAnswer {
                    test,
                    stdout,
                    stderr,
                });
            }
            Ok(result)
        }
        Ok(Err(err)) => {
            eprintln!("{err} {} {}", file!(), line!());
            Err(Output::ServerError)
        }
        Err(_) => Err(Output::Timeout(test)),
    }
}
