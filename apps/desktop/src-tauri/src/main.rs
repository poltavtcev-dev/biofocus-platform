//! BioFocus desktop binary entrypoint.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

fn main() {
    if let Err(error) = desktop_lib::run() {
        eprintln!("BioFocus desktop failed to start: {error}");
        std::process::exit(1);
    }
}
