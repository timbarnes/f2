// Read tokens from a file or stdin, one line at a time.
// Return one space-delimited token at a time.
// Cache the remainder of the line.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

use crate::messages::{DebugLevel, Msg};

#[derive(Debug)]
enum Source {
    Stdin,
    Stream(BufReader<File>),
}

enum FileMode {
    RW,
    RO,
}

pub struct FileHandle {
    pub source: Source, // Stdin or a file
    pub file_mode: FileMode,
    pub file_size: usize,
    pub file_position: usize,
    msg: Msg,
}

/// Reader handles input, from stdin or files
impl FileHandle {
    /// Reader::new creates a new stream handle
    ///
    pub fn new(file_path: Option<&std::path::PathBuf>, msg_handler: Msg) -> Option<FileHandle> {
        // Initialize a tokenizer.
        let mut message_handler = Msg::new();
        message_handler.set_level(DebugLevel::Error);
        match file_path {
            None => Some(FileHandle {
                source: Source::Stdin,
                file_mode: FileMode::RW,
                file_size: 0,
                file_position: 0,
                msg: msg_handler,
            }),
            Some(filepath) => {
                let file = File::open(filepath);
                match file {
                    Ok(file) => Some(FileHandle {
                        source: Source::Stream(BufReader::new(file)),
                        file_mode: FileMode::RW,
                        file_size: 0,
                        file_position: 0,        
                        msg: msg_handler,
                    }),
                    Err(_) => {
                        msg_handler.error(
                            "Reader::new",
                            "File not able to be opened",
                            Some(file_path),
                        );
                        None
                    }
                }
            }
        }
    }

    /// get_line returns a line of text from the input stream, or an error if unable to do so
    ///
    pub fn get_line(&mut self) -> Option<String> {
        // Read a line, storing it if there is one
        // In interactive (stdin) mode, blocks until the user provides a line.
        // Returns Option(line text). None indicates the read failed.
        let mut new_line = String::new();
        let result;
        match self.source {
            Source::Stdin => {
                io::stdout().flush().unwrap();
                result = io::stdin().read_line(&mut new_line);
            }
            Source::Stream(ref mut file) => result = file.read_line(&mut new_line),
        }
        match result {
            Ok(chars) => {
                if chars > 0 {
                    Some(new_line)
                } else {
                    None
                }
            }
            Err(e) => {
                self.msg
                    .error("get_line", "read_line error", Some(e.to_string()));
                None
            }
        }
    }

    /// read_char gets a single character from the input stream
    ///     Unfortunately it blocks until the user types return, so it can't be used
    ///     for truly interactive operations without a more complex implementation
    ///
    pub fn read_char(&self) -> Option<char> {
        let mut buf = [0; 1];
        let mut handle = io::stdin().lock();
        let bytes_read = handle.read(&mut buf);
        match bytes_read {
            Ok(_size) => Some(buf[0] as char),
            Err(_) => None,
        }
    }
}
