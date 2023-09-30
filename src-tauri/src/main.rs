// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use pyo3::prelude::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![invoke_py])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn invoke_py() -> Result<Vec<i64>, ()> {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let result = py
            .eval("[i * 10 for i in range(5)]", None, None)
            .map_err(|e| {
                e.print_and_set_sys_last_vars(py);
            })?;
        let res: Vec<i64> = result.extract().unwrap();
        Ok(res)
    })
}
