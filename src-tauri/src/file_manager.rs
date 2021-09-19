use std::fs::{self, File};

use crate::game_scanner::ScanResult;

/// Handles reading and writing scan results to disk
pub struct FileManager {}

impl FileManager {
  /**
  ### Summary
  Read the entire cached data file as a [`Vec`]<[`ScanResult`]>.

  These are ordered by default, as any new entries are appended to
  the end of the file, but no sorting is done.
  */
  pub fn read_data_from_disk() -> Vec<ScanResult> {
    let json_string = fs::read_to_string("data.json");
    let json_string = match json_string {
      Ok(string) => string,
      Err(_) => {
        // File couldn't be read, create it and return empty string
        File::create("data.json").expect("Impossibru");
        return vec![];
      }
    };

    // TODO: Fix this lazy boi, something COULD happen...
    // Apparently this is faster than file->Buffer->serde   -   https://github.com/serde-rs/json/issues/160
    let results: Vec<ScanResult> = serde_json::from_str(json_string.as_str()).expect("Lazy");
    return results;
  }

  /// Write a vector of ScanResults to the disk
  pub fn write_data_to_disk(item: ScanResult) {
    let mut existing = FileManager::read_data_from_disk();
    existing.push(item);
    let json_string = serde_json::to_string(&existing).expect("Couldn't convert");
    fs::write("data.json", json_string).expect("Couldn't write file");
  }
}
