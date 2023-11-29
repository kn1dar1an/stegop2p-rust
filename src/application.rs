extern crate libc;

use crate::application::tui::Tui;
use std::io;
use tokio::runtime::{Builder, Runtime};

mod action;
mod error;
mod event;
mod tui;

pub struct Application {
    async_runtime: Runtime,
}

impl Application {
    pub fn new() -> io::Result<Self> {
        let async_runtime = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .thread_name("StegoP2P")
            .thread_stack_size(3 * 1024 * 1024)
            .build()?;

        Ok(Self { async_runtime })
    }

    pub fn run(&self) -> io::Result<()> {
        let tui = Tui::new(self.async_runtime.handle().clone())?;

        let _ = self.async_runtime.block_on(tui.run());

        Ok(())
    }
    
    pub fn destroy(&self) {}
}

trait UiComponent {
    fn refresh(&self);
    fn refresh_no_update(&self);
}
