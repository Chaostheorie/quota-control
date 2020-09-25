mod ui;
use ansi_term::Colour::Red;
use std::{error::Error, result::Result};

fn main() -> Result<(), Box<dyn Error>> {
    // verification of ÓS and permissions (group based)
    if cfg!(windows) {
        // there's no support for windows at the moment
        // this is due to quota-control taking advantage of /dev/stdout and the users crate
        println!("There is currently no support for windows!");
        std::process::exit(9009);
    } else if !ui::backend::verify_privileges() {
        println!(
            "{} /home/quotas doesn't exist or is not a valid folder",
            Red.paint("Error:")
        );
        ui::backend::exit(5);
    }

    // render
    ui::interface::render()?;

    // exit
    ui::backend::exit(0);

    Ok(())
}
