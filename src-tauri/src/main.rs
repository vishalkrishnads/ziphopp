// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use ziphopp::{core::{Success, Error, open}, db::{Database, History}};

// just a state to pass the database between threads
struct HoppState(Mutex<Database>);

fn main() {
  tauri::Builder::default()
  .manage(HoppState(Mutex::new(Database::new("hopp.db", 5))))
  .invoke_handler(tauri::generate_handler![open_file, refresh])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

// command to open a file & add entry to recents list, if opened
// prompts user to choose a file if no path is provided
// opens with password if one is provided
// returns the result of operation
#[tauri::command]
fn open_file(path: Option<String>, password: Option<String>, state: tauri::State<HoppState>) -> Result<Success, Error> {
  let result = open(path, password);

  // accquire lock on the database
  if let Ok(mut db) = state.0.lock() {

    // if the operation was a success, this should be added to the recents list
    if let Ok(path) = &result {
      db.insert(&path.path).unwrap();
    }
  };

  result
}

// command to get the latest version of the recents list
// returns a result with vector representation of the queue in memory
#[tauri::command]
fn refresh(state: tauri::State<HoppState>) -> Result<History, Error> {
  // accquire a lock on the database
  if let Ok(db) = state.0.lock() {
    // get & return the recents list
    let result = db.refresh();
    Ok(result)
  } else { Err(Error::blank()) }
}