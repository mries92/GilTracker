use core::result::Result;
use std::fs::File;
use std::io::BufReader;

use crate::game_scanner::ScanResult;
use crate::error::{Error};
use crate::read_data_from_file;

/// Handles reading and writing scan results to disk
pub struct FileManager {
}

impl FileManager {
    /**
    ### Summary
    Read the entire cached data file as a [`Vec`]<[`ScanResult`]>.

    These are ordered by default, as any new entries are appended to
    the end of the file, but no sorting is done.

    ### Error cases
    - Data file has not yet been created, or has been deleted.
    - Data read from file could not be serialized to [`ScanResult`]
    
    ### Potential Errors
    - [`Error::DataFileReadError`]
    - [`Error::DataFileNotFound`]
    */
    pub fn read_data_from_disk() -> Result<Vec<ScanResult>, Error> {
        let file = File::open("data.json");
        let file = match file {
            Ok(file) => file,
            Err(_err) => return Err(Error::DataFileNotFound)
        };
        let reader = BufReader::new(file);
        let scan_results: Result<Vec<ScanResult>,_> = serde_json::from_reader(reader);
        let scan_results = match scan_results {
            Ok(scan_results) => scan_results,
            Err(_err) => return Err(Error::DataFileReadError)
        };
        Ok(scan_results)
    }

    /// Append a data entry to the file on disk
    pub fn write_data_to_disk(data: ScanResult) {
        let data = read_data_from_file();
    }
}