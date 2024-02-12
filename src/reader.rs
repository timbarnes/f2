// Provide read-line, key, key? emit and cr
//
// Prompts can be implemented in forth
// Minimize error handling and messaging.

use crate::messages::Msg;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

// use crate::messages::{DebugLevel, Msg};

#[derive(Debug)]
enum Source {
    Stdin,
    Stream(BufReader<File>),
}

pub struct Reader {
    source: Source, // Stdin or a file
    msg: Msg,
}

impl fmt::Debug for Reader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tokenizer").field(&self.source).finish()
    }
}

impl Reader {
    pub fn new(
        file_path: Option<&std::path::PathBuf>,
        // cont_prompt: &str,
        msg: Msg,
    ) -> Option<Reader> {
        //message_handler.set_level(DebugLevel::Error);
        match file_path {
            None => Some(Reader {
                source: Source::Stdin,
                msg,
            }),
            Some(filepath) => {
                let file = File::open(filepath);
                match file {
                    Ok(file) => Some(Reader {
                        source: Source::Stream(BufReader::new(file)),
                        // cont_prompt: cont_prompt.to_owned(),
                        msg,
                    }),
                    Err(_) => {
                        msg.error("Reader::new", "File not able to be opened", Some(file_path));
                        None
                    }
                }
            }
        }
    }

    pub fn read_line(&mut self) -> Result<String, &str> {
        // Read a line, storing it if there is one
        // In interactive (stdin) mode, blocks until the user provides a line.
        // Returns Ok(line text). None indicates the read failed.
        let mut new_line = String::new();
        match self.source {
            Source::Stdin => {
                // Issue prompt
                print!("Ok> ");
                io::stdout().flush().unwrap();
                // Read from Stdin
                match io::stdin().read_line(&mut new_line) {
                    Ok(_) => {
                        self.msg
                            .debug("get_line", "Got some values", Some(&new_line));
                        Ok(new_line)
                    }
                    Err(error) => {
                        self.msg
                            .error("get_line", "read_line error", Some(error.to_string()));
                        Err("read_line error")
                    }
                }
            }
            Source::Stream(ref mut file) => {
                // Read from a file. TokenSource is a BufReader. No prompts
                self.msg
                    .debug("get_line", "Reading from file", None::<bool>);
                let chars_read = &file.read_line(&mut new_line);
                match chars_read {
                    Ok(chars) => {
                        if *chars > 0 {
                            Ok(new_line)
                        } else {
                            Err("No more lines")
                        }
                    }
                    Err(_) => Err("read_line error"),
                }
            }
        }
    }

    pub fn key(&self) -> Result<u8, String> {
        // get a single character from the input line
        let mut buf = [0; 1];
        let mut handle = io::stdin().lock();
        let bytes_read = handle.read(&mut buf);
        match bytes_read {
            Ok(_size) => Ok(buf[0].clone()),
            Err(_) => Err("No characters read".to_owned()),
        }
    }

    pub fn key_available(&self) -> bool {
        // KEY? return true if there's a key available
        let mut bytes = io::stdin().bytes();
        if let Some(Ok(_)) = bytes.next() {
            true
        } else {
            false
        }
    }
}
