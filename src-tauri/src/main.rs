// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use ziphopp::{core::{Success, Error, open}, db::{Database, History}};

struct HoppState(Mutex<Database>);

fn main() {
  tauri::Builder::default()
  .manage(HoppState(Mutex::new(Database::new("hopp.db", 5))))
  .invoke_handler(tauri::generate_handler![open_file, refresh])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn open_file(path: Option<String>, password: Option<String>, state: tauri::State<HoppState>) -> Result<Success, Error> {
  let result = open(path, password);
  if let Ok(mut db) = state.0.lock() {
    if let Ok(path) = &result {
      db.insert(&path.path).unwrap();
    }
  };

  result
}

#[tauri::command]
fn refresh(state: tauri::State<HoppState>) -> Result<History, Error> {
  if let Ok(db) = state.0.lock() {
    let result = db.refresh();
    Ok(result)
  } else { Err(Error::blank()) }
}