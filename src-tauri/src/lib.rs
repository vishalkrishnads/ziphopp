pub mod core {
    use std::{path::{PathBuf, Path}, fs};
    use native_dialog::FileDialog;
    use zip::{ZipArchive, result::ZipError};

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
    pub struct Success {
        pub contents: Vec<String>,
        pub path: String
    }

    pub fn open(file: Option<String>, password: Option<String>) -> Result<Success, Error> {
        let path_buf = match file {
            Some(path) => Some(PathBuf::from(path)),
            None => FileDialog::new()
                        .show_open_single_file()
                        .expect("Failed to open file dialog")
        };

        if let Some(path) = path_buf {
            if let Ok(file) = fs::File::open(Path::new(&path)) {
                if let Ok(mut archive) = ZipArchive::new(file) {

                    // both the .by_index() and .by_index_decrypt() methods take a mutable &mut self reference of the object `archive`
                    // in order to avoid a .clone() call, we'll simply extract all the file paths beforehand
                    let mut contents = Vec::new();
                    for each in archive.file_names() {
                        contents.push(String::from(each));
                    }

                    // now, since the application requires the user to enter a password to unlock the .zip,
                    // we'll use the following block to do so & return the result if it works.
                    match password {
                        Some(password) => {
                            // once the password has been entered, this will run
                            
                            // descrypt the file to see if password is correct
                            match archive.by_index_decrypt(0, password.as_bytes()) {
                                Ok(zip) => {
                                    if let Ok(path) = path.into_os_string().into_string() {    
                                        match zip {
                                            Ok(_) => Ok(Success { contents, path }),
                                            Err(e) => {
                                                
                                                    Err(Error {
                                                        password_required: true,
                                                        path,
                                                        message: e.to_string()
                                                    })
                                            }
                                        }
                                    } else { Err(Error::blank()) }
                                },
                                Err(_) => Err(Error { password_required: false, path: String::from(""), message: String::from("") })
                            }
                        },
                        None => {
                            // this block would run when the user first selects a file
                            if let Ok(path) = path.into_os_string().into_string() {
                                // open the first file to see if it works.
                                match archive.by_index(0) {
                                    Ok(_) =>  Ok(Success { contents, path }),
                                    Err(error) => {
                                        
                                        let (password_required, message) = match error {
                                            ZipError::UnsupportedArchive(e) => (e == ZipError::PASSWORD_REQUIRED, String::from(e)),
                                            ZipError::InvalidArchive(e) => (false, String::from(e)),
                                            _ => (false, String::from(""))
                                        };
                                        Err(Error {
                                            password_required,
                                            path,
                                            message
                                        })
                                    }
                                }
                            } else { Err(Error::blank()) }
                        }
                    }
                } else { Err(Error::blank()) }
            } else { Err(Error::blank()) }
        } else { Err(Error::blank()) }
    } 
}

pub mod db {
    use std::collections::VecDeque;
    use std::fs::{File, OpenOptions};
    use std::io::{self, BufRead, BufReader, Write, Seek};
    use std::path::Path;

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

        // utility function to get the file name
        fn get_filename(&self, path: &str) -> String {
            let path_obj = Path::new(path);
            if let Some(file_name) = path_obj.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    return file_name_str.to_string();
                }
            }
            path.to_string() // Return the original path if extraction fails
        }

        pub fn refresh(&self) -> History {
            let mut history = Vec::new();

            for value in &self.queue {
                history.push(HoppFile {
                    name: self.get_filename(&value),
                    path: value.clone()
                });
            }

            History { history }
        }
    }

}    