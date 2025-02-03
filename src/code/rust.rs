use crate::Output;

pub async fn rust_compile(code: &str) -> Result<Vec<u8>, Output> {
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().map_err(|_| Output::ServerError)?;
    let wasm = NamedTempFile::new().map_err(|_| Output::ServerError)?;
    writeln!(file, "{}", code).map_err(|_| Output::ServerError)?;
    let output = std::process::Command::new("rustc")
        .args([
            file.path().to_str().ok_or(Output::ServerError)?,
            "--target=wasm32-wasi",
            "-o",
            wasm.path().to_str().ok_or(Output::ServerError)?,
        ])
        .output()
        .map_err(|_| Output::ServerError)?;
    if !output.status.success() {
        return Err(Output::CannotCompile(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    std::fs::read(wasm.path()).map_err(|_| Output::ServerError)
}
