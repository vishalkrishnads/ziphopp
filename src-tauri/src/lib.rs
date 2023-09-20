use std::path::Path;

// utility function to get the file name
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

    #[derive(Clone, serde::Serialize)]
    pub struct Error {
        pub password_required: bool,
        pub path: String,
        pub message: String
    }

    impl Error {
        pub fn blank() -> Self {
            Error {
                password_required: false,
                path: String::from(""),
                message: String::from("")
            }
        }
    }

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
        pub contents: Vec<String>,
        pub path: String,
        pub meta: MetaData
    }

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
        };
    
        let path = path_buf
            .as_ref()
            .and_then(|path| path.to_str())
            .ok_or_else(|| Error::blank())?;
    
        let file = fs::File::open(&path).map_err(|e| Error {
            password_required: false,
            path: String::from(""),
            message: e.to_string(),
        })?;
    
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

    pub struct Database {
        file: File,
        max_entries: usize,
        queue: VecDeque<String>,
    }

    #[derive(Clone, serde::Serialize)]
    struct HoppFile {
        name: String,
        path: String
    }

    #[derive(Clone, serde::Serialize)]
    pub struct History {
        history: Vec<HoppFile>
    }

    impl Database {
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

        fn load_from_file(&mut self) -> io::Result<()> {
            let reader = BufReader::new(&self.file);

            for line in reader.lines() {
                let line = line?;
                self.queue.push_back(line);
            }
            Ok(())
        }

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

        fn save_to_file(&mut self) -> io::Result<()> {
            self.file.set_len(0).unwrap();
            self.file.seek(std::io::SeekFrom::Start(0)).unwrap(); // Reposition cursor to the start.

            for data in &self.queue {
                writeln!(&self.file, "{}", data)?;
            }

            self.file.flush()?;
            Ok(())
        }

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