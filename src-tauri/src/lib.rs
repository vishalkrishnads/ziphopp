use std::path::Path;

/// This function gets the name of a file given a path
/// 
/// # Arguments
/// 
/// * `path` - The file path as &str
/// 
/// # Returns
/// 
/// The name of the file in the filesystem
fn get_filename(path: &str) -> String {
    let path_obj = Path::new(path);
    if let Some(file_name) = path_obj.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            return file_name_str.to_string();
        }
    }
    path.to_string() // Return the original path if extraction fails
}

pub mod core {
    use std::{path::PathBuf, fs};
    use native_dialog::FileDialog;
    use zip::{ZipArchive, result::ZipError};
    use super::get_filename;

    /// Represents an error that can occur when a ZIP is opened
    #[derive(Clone, serde::Serialize)]
    pub struct Error {
        /// This field indicates whether the error is related either to the password being invalid, or the file requiring a password for decryption
        pub password_required: bool,
        /// The file path to the archive file
        pub path: String,
        /// A verbose message about the error itself
        pub message: String
    }

    impl Error {
        /// Creates a new `Error` instance with default values.
        pub fn blank() -> Self {
            Error {
                password_required: false,
                path: String::from(""),
                message: String::from("")
            }
        }
    }

    /// Represents the metadata about a ZIP archive
    #[derive(Clone, serde::Serialize)]
    pub struct MetaData {
        /// The size of the archive when compressed, formatted as a string. It displays the size for example, as x.x GB
        compressed: String,
        /// The size of the archive when uncompressed, formatted as a string. It displays the size for example, as x.x GB
        size: String,
        /// The name of the archive file
        name: String
    }

    impl MetaData {

        /// Formats a byte size into a human-readable string with appropriate units (B, KB, MB or GB)
        fn format_bytes(bytes: u64) -> String {

            // an array to hold the different units
            const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
            let mut value = bytes as f64; // make a mutable copy of the value
            let mut unit_idx = 0; // idx tracks which unit to use. it starts from B
        
            // we'll divide value by 1024 bytes until it's either less than a single byte (in which case we can safely assume it's B)
            // or the idx goes above safe range (ie, above 3 would result in it goind out of index of UNITS)
            while value >= 1024.0 && unit_idx < UNITS.len() - 1 {
                value /= 1024.0;
                unit_idx += 1;
            }
        
            format!("{:.2}{}", value, UNITS[unit_idx])
        }

        /// Creates a new `MetaData` instnace with the specified properties
        fn new(compressed: u64, size: u64, name: String) -> Self {
            MetaData { compressed: Self::format_bytes(compressed), size: Self::format_bytes(size), name }
        }
    }

    /// Represents the result of a successful parse operation
    #[derive(Clone, serde::Serialize)]
    pub struct MetaData {
        compressed: String,
        size: String,
        name: String
    }

    impl MetaData {
        fn format_bytes(bytes: u64) -> String {

            // an array to hold the different units
            const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
            let mut value = bytes as f64; // make a mutable copy of the value
            let mut unit_idx = 0; // idx tracks which unit to use. it starts from B
        
            // we'll divide value by 1024 bytes until it's either less than a single byte (in which case we can safely assume it's B)
            // or the idx goes above safe range (ie, above 3 would result in it goind out of index of UNITS)
            while value >= 1024.0 && unit_idx < UNITS.len() - 1 {
                value /= 1024.0;
                unit_idx += 1;
            }
        
            format!("{:.2}{}", value, UNITS[unit_idx])
        }

        fn new(compressed: u64, size: u64, name: String) -> Self {
            MetaData { compressed: Self::format_bytes(compressed), size: Self::format_bytes(size), name }
        }
    }

    #[derive(Clone, serde::Serialize)]
    pub struct Success {
        /// A veector containing the file paths of everything in the archive, when uncompressed
        pub contents: Vec<String>,
        /// The file path of the zip archive
        pub path: String,
        /// Metadata of the file as stated in the requirements
        pub meta: MetaData
    }

    /// Opens a ZIP archive specified by the `file` path and optional `password`.
    ///
    /// If `file` is `None`, a file dialog is displayed to select the archive.
    ///
    /// # Arguments
    ///
    /// * `file` - An optional path to the ZIP archive.
    /// * `password` - An optional password to decrypt the archive.
    ///
    /// # Returns
    ///
    /// A `Result` containing either a `Success` or an `Error`.
    pub fn open(file: Option<String>, password: Option<String>) -> Result<Success, Error> {
        let path_buf = match file {
            Some(path) => Some(PathBuf::from(path)),
            None => FileDialog::new()
                .add_filter("ZIP Archive", &["zip"])
                .show_open_single_file()
                .map_err(|e| Error {
                    password_required: false,
                    path: String::from(""),
                    message: e.to_string(),
                })?,
        }; // if there is a path, use it or else just prompt the user to choose one
    
        // extract the path value as &str
        let path = path_buf
            .as_ref()
            .and_then(|path| path.to_str())
            .ok_or_else(|| Error::blank())?;
    
        // try opening the file at the path. if it fails, return the error
        let file = fs::File::open(&path).map_err(|e| Error {
            password_required: false,
            path: String::from(""),
            message: e.to_string(),
        })?;
    
        // read the zip file as a ZipArchive to work with it
        let mut archive = ZipArchive::new(file).map_err(|e| Error {
            password_required: false,
            path: path.to_string(),
            message: e.to_string(),
        })?;

        let mut contents = Vec::new();
        for each in archive.file_names() {
            contents.push(String::from(each));
        }

        let name = get_filename(path);
        let path = String::from(path);
        let mut compressed: u64 = 0;
        let mut size: u64 = 0;
        // now, since the application requires the user to enter a password to unlock the .zip,
         // we'll use the following block to do so & return the result if it works.
        match password {
            Some(password) => {
                // once the password has been entered, this will run
                
                // descrypt the file to see if password is correct
                match archive.by_index_decrypt(0, password.as_bytes()) {
                    Ok(zip) => {  
                        match zip {
                            Ok(file) => {
                                size += file.size();
                                compressed += file.compressed_size();
                            },
                            Err(e) => {
                                return Err(Error {
                                    password_required: true,
                                    path,
                                    message: e.to_string()
                                })
                            }
                        }     
                    },
                    Err(_) => return Err(Error::blank())
                }
            },
            None => {
                // if there is no password, this block will run

                for i in 0..archive.len() {
                    match archive.by_index(i) {
                        Ok(file) =>  {
                            size += file.size();
                            compressed += file.compressed_size();
                        },
                        Err(error) => {                    
                            let (password_required, message) = match error {
                                ZipError::UnsupportedArchive(e) => (e == ZipError::PASSWORD_REQUIRED, String::from(e)),
                                ZipError::InvalidArchive(e) => (false, String::from(e)),
                                _ => (false, String::from(""))
                            };
                            return Err(Error {
                                password_required,
                                path,
                                message
                            })
                        }
                    }
                }
            }
        };

        Ok(Success { contents, path, meta: MetaData::new(compressed, size, name) })
    }
    
}

pub mod db {
    use std::collections::VecDeque;
    use std::fs::{File, OpenOptions};
    use std::io::{self, BufRead, BufReader, Write, Seek};
    use super::get_filename;

    /// The database of the application to store recently opened files
    pub struct Database {
        /// The file which is used as the permenant backup storage
        file: File,
        /// The maximum no of recent files displayed, the requirements state it to be 5
        max_entries: usize,
        /// The queue in memory for fast access at runtime
        queue: VecDeque<String>,
    }

    /// Represents a single ZIP file opened recently
    #[derive(Clone, serde::Serialize)]
    struct HoppFile {
        /// The name of the ZIP arhive
        name: String,
        /// The file path of the ZIP archive
        path: String
    }

    /// Represents the recent history. Used to return a result to the front-end
    #[derive(Clone, serde::Serialize)]
    pub struct History {
        /// A vector containing 5 recently opened files
        history: Vec<HoppFile>
    }

    impl Database {
        /// Returns a new instance of `Database`
        /// 
        /// # Arguments
        /// 
        /// * `file_path` - The file path as &str
        /// * `max_entries` - The maximum number of entries the database should hold
        /// 
        /// # Returns
        /// 
        /// A new instance of `Database` with the given parameters
        pub fn new(file_path: &str, max_entries: usize) -> Self {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(file_path).unwrap();

            let mut database = Database {
                file,
                max_entries,
                queue: VecDeque::new(),
            };

            // Load existing data from the file, if it exists.
            database.load_from_file().unwrap();
            database
        }

        /// Loads all the existing data in the file into memory
        fn load_from_file(&mut self) -> io::Result<()> {
            // create a BufReader from the file
            let reader = BufReader::new(&self.file);

            for line in reader.lines() {
                let line = line?;
                self.queue.push_back(line);
            }
            Ok(())
        }

        /// Inserts a new entry into the database
        /// 
        /// # Arguments
        /// 
        /// * `data` - The data to insert as str
        /// 
        /// # Returns
        /// 
        /// A `Result<>` indicating the result of the operation
        pub fn insert(&mut self, data: &str) -> io::Result<()> {
            // Check if the data (file) already exists in the queue.
            if let Some(existing_index) = self.queue.iter().position(|item| item == data) {
                // If it exists, remove it from the queue.
                self.queue.remove(existing_index);
            } else if self.queue.len() >= self.max_entries {
                // If the queue is full, remove the oldest entry.
                self.queue.pop_front();
            }
        
            // Push the data (file) to the front of the queue.
            self.queue.push_front(data.to_string());
        
            // Save the updated data to the file.
            self.save_to_file()?;
            Ok(())
        }        

        /// Saves a copy of the queue to the file
        fn save_to_file(&mut self) -> io::Result<()> {
            self.file.set_len(0).unwrap();
            // Reposition cursor to the start.
            self.file.seek(std::io::SeekFrom::Start(0)).unwrap(); 

            for data in &self.queue {
                writeln!(&self.file, "{}", data)?;
            }

            // flush() is used to immediately flush the data onto the file
            self.file.flush()?;
            Ok(())
        }

        /// Returns all the contents of the database for the front end to refresh recents list
        /// 
        /// # Returns
        /// 
        /// An instance of `History` containing a vector with the database contents
        pub fn refresh(&self) -> History {
            let mut history = Vec::new();

            for value in &self.queue {
                history.push(HoppFile {
                    name: get_filename(&value),
                    path: value.to_string()
                });
            }

            History { history }
        }
    }
}    