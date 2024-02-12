// system configuration and command line processing

use crate::f2::F2;
use ::clap::{arg, Command};

const VERSION: &str = "alpha.24.2.9";
const WELCOME_MESSAGE: &str = "Welcome to f2.";
const EXIT_MESSAGE: &str = "Finished";
const DEFAULT_CORE: [&str; 2] = ["~/.f2/corelib.fs", "src/f2.fs"];

pub struct Config {
    // debug_level: DebugLevel,
    loaded_file: String,
    loaded_core: bool,
    core_file: String,
    no_core: bool,
    pub run: bool,
    forth: F2,
}

impl Config {
    pub fn new() -> Result<Config, String> {
        let fth = F2::new();
        match fth {
            Ok(f2) => Ok(Config {
                // debug_level: DebugLevel::Error,
                loaded_file: "".to_owned(),
                loaded_core: false,
                core_file: DEFAULT_CORE[0].to_owned(),
                no_core: false,
                run: true,
                forth: f2,
            }),
            Err(e) => Err(format!("Failed to initialize f2: {}", e).to_owned()),
        }
    }

    pub fn process_args(&mut self) -> &Config {
        // process arguments
        // let msg = Msg::new(); // Create a message handler for argument errors

        let arguments = Command::new("tForth")
            .version(VERSION)
            .author("Tim Barnes")
            .about("A simple Forth interpreter")
            .arg(
                arg!(--debuglevel <VALUE>)
                    .required(false)
                    .value_parser(["error", "warning", "info", "debug"]),
            )
            .arg(arg!(-l --library <VALUE>).required(false))
            .arg(arg!(-f --file <VALUE>).required(false))
            .arg(arg!(-n - -nocore).required(false))
            .get_matches();

        let library = arguments.get_one::<String>("library");
        if let Some(lib) = library {
            self.core_file = lib.to_string();
        }

        let nocore = arguments.get_one::<bool>("nocore");
        if let Some(nc) = nocore {
            self.no_core = *nc;
        }

        let file = arguments.get_one::<String>("file");
        if let Some(file) = file {
            self.loaded_file = file.clone();
        }
        self
    }

    pub fn run_forth(&mut self) {
        // create and run the interpreter
        // return when finished

        let f2 = F2::new();
        match f2 {
            Ok(fth) => {
                let mut forth = fth;
                forth.init();

                if !self.no_core {
                    for path in DEFAULT_CORE {
                        if forth.f2_load_file(&path.to_owned()) {
                            break;
                        }
                    }
                }
                if self.loaded_file != "" {
                    if !forth.f2_load_file(&self.loaded_file) {
                        println!("Couldn't load file {}", &self.loaded_file);
                    }
                }

                // forth.set_abort_flag(false); // abort flag may have been set by load_file, but is no longer needed.

                println!("{WELCOME_MESSAGE} Version {VERSION}");

                // Enter the interactive loop to read and process input
                loop {
                    if forth.should_exit() {
                        println!("{EXIT_MESSAGE}");
                        break;
                    }

                    // Process one word (in immediate mode), or one definition (compile mode).
                    forth.run();
                }
            }
            Err(e) => println!("{}", e),
        }
        // forth.msg.set_level(self.debug_level.clone());
    }

    pub fn exit(&self) {
        println!("{EXIT_MESSAGE}");
    }
}
