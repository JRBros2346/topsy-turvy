use crate::Output;

pub async fn cpp_compile(code: &str) -> Result<Vec<u8>, Output> {
    use std::fs;
    let dir = tempfile::tempdir().map_err(|_| Output::ServerError)?;
    let file = dir.path().join("main.cpp");
    fs::write(&file, code).map_err(|_| Output::ServerError)?;
    let output = std::process::Command::new("clang++")
        .current_dir(dir.path())
        .args(["--target=wasm32-wasi", "main.cpp", "-o", "main.wasm"])
        .output()
        .map_err(|_| Output::ServerError)?;
    if !output.status.success() {
        return Err(Output::CannotCompile(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    std::fs::read(dir.path().join("main.wasm")).map_err(|_| Output::ServerError)
}
