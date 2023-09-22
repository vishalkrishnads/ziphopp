# `ZipHopp` 🛸
A desktop app that lists the contents of a ZIP archive, built using the [Tauri framework](https://tauri.app). It lets you select a .zip file from your PC, and lists it's contents by decrypting it. Download the file corresponding to your OS from the [releases page](https://github.com/vishalkrishnads/ziphopp/releases) to use it right away.

> :warning: This is the submission for an interview task assigned to me as part of the **Rust Engineer** role at [HoppScotch](https://hoppscotch.io), and not one of my projects.

## Table Of Contents
* [Problem Statement](#problem-statement)
    * [Functionality Required](#functionality-required)
    * [Constraints & Nuances](#constraints--nuances)
* [Implementation](#implementation)
    * [ZIP handling](#zip-handling)
        * [Obtaining file from the user](#obtaining-file-from-the-user)
        * [Decrypting the file](#decrypting-the-file)
    * [Recent files](#recent-files)
    * [The UI](#the-ui)
    * [API & Communication](#api--communication)
        * [Opening files](#opening-files)
        * [Recents list](#recents-list)
    * [Control Flow](#control-flow)
* Setup Guide
    * Prerequisites
    * Build from source
    * Documentation

## Problem Statement
In the assigned task, the candidate is required to build a ZIP file viewer that can view the contents of a ZIP file in the system. They are to write it using the [Tauri framework](https://tauri.app). The full document detailing the task can be found [here]() for reference.

### Functionality Required
1. The app should provide a window where the user can click to select and open a ZIP
file.
2. Once a ZIP file is selected, the app should parse the ZIP file and show a tree
showing all the files in the ZIP file (just the path).
3. The app should handle ZIP files locked with a password and ask the user the
password (along with its validation) and then show the files to the user.
4. The app should also show some metadata about the ZIP file itself (name, size,
compressed size etc).
5. The app should have a welcome screen where it remembers and shows the list of
last 5 ZIPs you have opened.

### Constraints & Nuances
1. The ZIP file handling part should be written in Rust.
2. Any frontend JS framework (or even vanilla) can be used to build the frontend/ui
portion of the app.
3. You should use Tauri Commands to communicate back and forth between the UI
and the Rust layer

## Implementation
This section details the implementation of the different functions inside the app for you to understand my thought process better while evaluating. The whole aim here is to get stuff done with minimal code, and I've tried to achieve it in this project.

### ZIP handling
The ZIP handling logic is preformed using Rust at the backend, as stated in the requirements. This however, consists of two parts: i. actually getting a file from the user elegantly, ii. opening & decrpyting the contents of the file itself.

#### Obtaining file from the user
For making the process of selecting a file intuitive and GUI based, I'm using the [`native-dialog::FileDialog`](https://docs.rs/native-dialog/latest/native_dialog/struct.FileDialog.html) to open up a file picker window with a filter to restrict users into choosing only zip archives. The path obtained from the dialog is then used to open the file itself. The relevant code for it can be found in `src-tauri/src/lib.rs` in `core::open()`.

```rust
    FileDialog::new()
        .add_filter("ZIP Archive", &["zip"])
        .show_open_single_file()
        .map_err(|e| Error {
            password_required: false,
            path: String::from(""),
            message: e.to_string(),
        })?
```

This code only runs if there is no path provided to `open()`. More about the API itself can be found below in the [API & Communication](#api--communication) section.

#### Decrypting the file
For decrypting the file itself, I'm using the [`zip`](https://docs.rs/zip/latest/zip/) crate, which has all the methods required for this application built in. The methods [`by_index()`](https://docs.rs/zip/latest/zip/read/struct.ZipArchive.html#method.by_index) & [`by_index_decrypt()`](https://docs.rs/zip/latest/zip/read/struct.ZipArchive.html#method.by_index_decrypt) are being used for non-encrypted & password encrypted files respectively. The logic is simple: the backend opens the zip archive, and uses these methods to loop over all files in it. On each iteration, it uses the methods [`compressed_size()`](https://docs.rs/zip/latest/zip/read/struct.ZipFile.html#method.compressed_size), [`size()`](https://docs.rs/zip/latest/zip/read/struct.ZipFile.html#method.size) & [`name()`](https://docs.rs/zip/latest/zip/read/struct.ZipFile.html#method.name) from [`zip::read::ZipFile`](https://docs.rs/zip/latest/zip/read/struct.ZipFile.html) to get it's metadata & name respectively and adds them to the overall result. At last, it just returns the result itself.

> Under the hood, it can be found that both `by_index()` & `by_index_decrypt()` are actually wrappers of a private method `by_index_with_optional_password()` with the following signature:
> ```rust
> fn by_index_with_optional_password<'a>(
>         &'a mut self,
>         file_number: usize,
>         mut password: Option<&[u8]>,
>     ) -> ZipResult<Result<ZipFile<'a>, InvalidPassword>>
> ```
> If this was declared public, there was a way to reduce code even more by directly calling the function and skipping the password checking logic in `open()`, thereby improving performance. But since it isn't, this is what I could come up with.

As such, the method signature of `open()` becomes 
```rust
    pub fn open(
        file: Option<String>, 
        password: Option<String>
    ) -> Result<Success, Error>
```

which can return a `Success` or `Error` state of the open operation. It can take in an optional password & an optional path. You can see more about the front-end API below in the [API & Communication](#api--communication) section.

#### Recent Files
The recently opened files are stored in volatile memory as a queue & in a file in non-volatile memory. Every time there is a change in the queue, the changes are written to disk to prevent data loss. The order of entries in the queue represent the order in which the files were opened by the user. The API exposes a method for the front-end to refresh the recents list, which essentially returns a vector containing all the entries in the file.

If you're wondering why I resorted to using a simple old plain text file for storing the recent file entries, the reason is that introducing a dedicated database in this application seems like a performance overhead & quite frankly, is a bit overkill for this application. Also, in the current implementation, the file itself can be removed from the setup to improve performance while still getting a volatile recents list.

### The UI
The UI itself is written in [Next.js](https://nextjs.org), which I mostly chose for it's ease of use over vanilla HTML, CSS & JS. Since the task document states that UI polish won't be judged here, I haven't put much care into building a proper UI. As such, if any files with long names are opened in the app, the UI might go haywire and look pretty bad, although none of the functionality will be compromised. Also, if you're a web dev reading the front end code, you 'll find that no best practices like separation of concerns or proper documentation are followed in the front-end code. This is because the task states that the focus is on the Rust code. However, the format of the API is detailed below so that you can understand the communication part better

### API & Communication
The front-end would have to communicate with the back-end for either of 2 purposes: either to open a zip file, or to get a list of all recently opened files. As such, the API exposes two [Tauri Commands](https://tauri.app/v1/guides/features/command/) for achieving this.

#### Opening files
The back-end exposes a command for opening a zip file and returning it's contents. It can be invoked from the front-end JS code like so

```javascript
    import { invoke } from '@tauri-apps/api'

    let payload = {};

    // if you wanna open with password
    payload.password = 'your_password';
    // if you wanna open a file using a path
    // useful for reopening recent files
    payload.path = 'your_path';

    invoke('open_file', payload)
    .then((result) => {
        // you'll get a successful response with the contents
    }).catch((error) => {
        // do something with the error
        // if it's a password related error, error.password_required will be true
    })
```

The function signature for this command is as you'd expect

```rust
    #[tauri::command]
    fn open_file(
        path: Option<String>, 
        password: Option<String>,
        // for accessing the shared database instance 
        state: tauri::State<HoppState>
        ) -> Result<Success, Error>
```
In summary, the back-end exposes a single function which you can call for doing all zip file related tasks, be it simply opening a file of the user's choice, decrypting a password encrpyted file or opening a file of your choice.

#### Recents list

Contrary to the file opening API, the recents list API is pretty simple to invoke as it doesn't take in any payloads. All it does is return a result, which if successful, will contain an array of the recently opened files, where each entry would contain a file name & path. Here's how it can be invoked

```javascript
    import { invoke } from '@tauri-apps/api'

    invoke('refresh')
    .then((result) => {
        for(const each of result.history) {
            console.log(`File ${each.name} is at ${each.path}`)
        }
    }).catch((error) => {
        // do something
    })
```

The function signature for this command is also pretty simple

```rust
    #[tauri::command]
    fn refresh(
        state: tauri::State<HoppState>
        ) -> Result<History, Error>
```

In summary, this command provides a simple interface for the front-end to refresh the list of recently opened files. I have tried to achieve maximum functionality with limited number of commands thus providing an elegant API for the front-end, and hence is the reason why all the file handling logic has been put into the single command. You can read more about the control flow between the front-end & back-end below.

> In an attempt to improve performance, I have avoided any deep copy operations using `.clone()` throughout the code. Nothing big, but just putting it out there in case you haven't noticed.😁

### Control Flow
This section details how the control flows within the application for different tasks. I intend to express my thought process here, although I'm not sure how well you'll be able to understand. If you're interested, please keep reading. If not, feel free to skip to the next section.

I'll detail the control flow of opening a new zip file in the application. As for the recents list, I don't feel like there's much to detail in terms of the communication part. So here goes

1. A user clicks the **Open File** button in the UI, thus invoking the `open_file` command with an empty payload `{}`. Since no path is provided, the file picker dialog pops up allowing the user to choose a file
2. The back-end will attempt opening and reading the file
    * If successful, it'll return a successful response along with the contents. It'll also add the file to the recents list.
    * If not, it'll return an error message which will also contain the path of the file the user selected
3. If the response is a successful one, the front-end will skip to step 5. If not, it will look at the error message to see if the `password_required` field is true. If so, it'll display a password popup and keep the file path in memory.
4. Once the popup is visible, the user can enter a password & when that happens, the front-end invokes the `open_file` command again, but this time, with a path & password in the payload. Again, for the response, it'll repeat step 3.
5. The successful result will have a contents array with the file paths of every file inside the archive. The front-end will display it and then invoke the `refresh` command to update the recents list.

As for the recents list, every time some entry is added to the in memory queue, it's also updated in the non-volatile backup storage (which currently, is a plain text file). When the `refresh` command is invoked, it returns a copy of the queue as a result. The database itself is handled as a managed state within the Tauri application, which is sybchronized by a mutex. The instance will have already opened the text file and will hold on to it. This helps reduce the I/O overhead of having to open & close the text file each time a change is made in the DB.