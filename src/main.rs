// f2 main program

mod config;
mod doc;
mod f2;
mod messages;
mod reader;

use config::Config;

fn main() {
    let config = Config::new();
    match config {
        Ok(cfg) => {
            let mut configuration = cfg;
            configuration.process_args();
            if configuration.run {
                configuration.run_forth();
            } else {
                configuration.exit();
            }
        }
        Err(e) => {
            println!("Failed to initialize: {}", e);
        }
    }
}
