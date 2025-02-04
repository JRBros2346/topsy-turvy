use core::time;
use std::{
    thread,
    time::{Duration, Instant},
};

use wasmer::{Instance, Module, Store};
use wasmer_wasi::WasiState;

use crate::Output;

pub async fn wasm_test(
    wasm: Vec<u8>,
    memory_limit: u32,
    time_limit: u64,
    tests: Vec<(Vec<String>, String)>,
) -> Output {
    let mut store = Store::default();
    let module = Module::new(&store, wasm).map_err(|e| Output::CannotCompile(e.to_string()))?;
    let mut wasi = WasiState::new("wasm")
        .finalize(&mut store)
        .map_err(|e| Output::CannotCompile(e.to_string()))?;
    let imports = wasi
        .import_object(&mut store, &module)
        .map_err(|e| Output::WasiError(e))?;
    let instance = Instance::new(&mut store, &module, &imports)
        .map_err(|e| Output::CannotCompile(e.to_string()))?;
    let memory = instance
        .exports
        .get_memory("memory")
        .map_err(|e| Output::CannotCompile(e.to_string()))?;
    memory
        .grow(&mut store, memory_limit / 64)
        .map_err(|e| Output::CannotCompile(e.to_string()))?;
    let func = instance
        .exports
        .get_function("main")
        .map_err(|e| Output::CannotCompile(e.to_string()))?;
    for test in tests {
        let start = Instant::now();
        let timeout = Arc::new(AtomicBool::new(false));
        let timeout_clone = Arc::clone(&timeout);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(time_limit));
            let mut timed_out = timeout_clone.lock().unwrap();
            *timed_out = true;
        });
    }
}
