use std::io;

use crate::application::Application;

pub mod application;

fn main() -> io::Result<()> {
    let application: Application = Application::new()?;

    application.run()?;

    Ok(())
}
