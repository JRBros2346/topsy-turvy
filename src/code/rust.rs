use crate::submit::{Output, TestCases};

pub async fn rust_run(code: &str, tests: TestCases) -> Output {
    use std::fs;
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt as _;
    use tokio::process::Command;
    use tokio::time::{Duration, Instant};
    let dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("{err} {} {}", file!(), line!());
            return Output::ServerError
        }
    };
    let file = dir.path().join("main.rs");
    if let Err(err) = fs::write(&file, code) {
        eprintln!("{err} {} {}", file!(), line!());
        return Output::ServerError;
    }
    let output = match Command::new("rustc")
        .current_dir(dir.path())
        .args(["main.rs", "-o", "main"])
        .output()
        .await
    {
        Ok(output) => output,
        Err(err) => {
            eprintln!("{err} {} {}", file!(), line!());
            return Output::ServerError;
        }
    };
    if !output.status.success() {
        return Output::CannotCompile(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let mut results = vec![];
    for test in tests.public {
        let mut child = match Command::new("./main")
            .current_dir(dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => {
                eprintln!("{err} {} {}", file!(), line!());
                return Output::ServerError;
            }
        };
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(err) = stdin.write_all(test.input.as_bytes()).await {
                eprintln!("{err} {} {}", file!(), line!());
                return Output::ServerError;
            }
        }
        let start = Instant::now();
        let execution =
            tokio::time::timeout(Duration::from_secs(5), child.wait_with_output()).await;
        let result = start.elapsed();
        match execution {
            Ok(Ok(output)) => {
                if !output.status.success() {
                    return Output::RuntimeError(
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    );
                }
                let out = String::from_utf8_lossy(&output.stdout).to_string();
                if out.trim() != test.output {
                    return Output::WrongAnswer(test, out);
                }
                results.push(result);
            }
            Ok(Err(err)) => {
                eprintln!("{err} {} {}", file!(), line!());
                return Output::ServerError;
            }
            Err(_) => return Output::Timeout(test),
        }
    }
    let mut child = match Command::new("./main")
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            eprintln!("{err} {} {}", file!(), line!());
            return Output::ServerError;
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(err) =stdin
            .write_all(tests.hidden.input.as_bytes())
            .await
        {
            eprintln!("{err} {} {}", file!(), line!());
            return Output::ServerError;
        }
    }
    let start = Instant::now();
    let execution = tokio::time::timeout(Duration::from_secs(5), child.wait_with_output()).await;
    let result = start.elapsed();
    match execution {
        Ok(Ok(output)) => {
            if !output.status.success() {
                return Output::RuntimeError(String::from_utf8_lossy(&output.stderr).to_string());
            }
            let out = String::from_utf8_lossy(&output.stdout).to_string();
            if out.trim() != tests.hidden.output {
                return Output::Hidden;
            }
            results.push(result);
        }
        Ok(Err(err)) => {
            eprintln!("{err} {} {}", file!(), line!());
            return Output::ServerError;
        }
        Err(_) => return Output::Timeout(tests.hidden),
    }
    let avg = results.iter().sum::<Duration>() / results.len() as u32;
    let error = match results.iter().map(|&r| r.abs_diff(avg)).max() {
        Some(error) => error,
        None => {
            eprintln!("No results");
            return Output::ServerError;
        }
    };
    Output::Accepted(avg, error)
}
