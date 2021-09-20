use rusqlite::{params, Connection, Result};
use std::{
  fs,
  sync::{Arc, Mutex},
};

use crate::game_scanner::ScanResult;

/// Manages an internal database for reading and writing data.
///
/// Operations on db context are performed asynchronously.
pub struct FileManager {
  connection: Arc<Mutex<Connection>>,
}

impl FileManager {
  pub fn new() -> FileManager {
    let path = "./data.db3";
    let fm = FileManager {
      connection: Arc::new(Mutex::new(Connection::open(&path).unwrap())),
    };
    let con = fm.connection.clone();
    let con = con.lock().unwrap();
    let _ = con.execute("
    CREATE TABLE ScanResults(
    id INTEGER PRIMARY KEY,
    gil INTEGER,
    mgp INTEGER,
    company_seals INTEGER)",[],);
    return fm;
  }

  /**
  ### Summary
  Read the entire cached data file as a [`Vec`]<[`ScanResult`]>.

  These are ordered by default, as any new entries are appended to
  the end of the file, but no sorting is done.
  */
  pub fn read_data_from_disk(&self) -> Vec<ScanResult> {
    let con = self.connection.lock().unwrap();
    let mut prep = con
      .prepare("SELECT * FROM ScanResults")
      .expect("Trouble preparing statement"); //TODO map this
    let iter = prep.query_map([], |row| {
      Ok(ScanResult {
        gil: row.get(1)?,
        mgp: row.get(2)?,
        company_seals: row.get(3)?,
        timestamp: row.get(4)?,
      })
    }).expect("Failed to map entity");

    let mut results: Vec<ScanResult> = vec![];
    for result in iter {
      results.push(result.unwrap());
    }

    return results;
  }

  /// Write a vector of ScanResults to the disk
  pub fn write_data_to_disk(&self, item: &ScanResult) {
    println!("Write data called");
    let con = self.connection.lock().unwrap();
    con
      .execute(
        "
    INSERT INTO ScanResults (gil, mgp, company_seals)
    VALUES (?1, ?2, ?3)",
        [item.gil, item.mgp, item.company_seals],
      )
      .unwrap();
  }
}
