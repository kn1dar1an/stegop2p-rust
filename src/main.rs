use std::{thread, time::Duration};

use async_runtime::AsyncRuntime;

use crate::async_runtime::async_task::AsyncTaskResult;

pub mod ui;
mod async_runtime;

fn main() {
    let runtime = AsyncRuntime::new();

    match runtime {
        Ok(mut rt) => {
            rt.start().unwrap();
            let spawner = rt.get_spawner().unwrap();
            let mut count = 0;
            loop {
                if count == 5 { break; }
                let result = spawner.spawn_non_blocking( async {
                    println!("Hello!");
                    AsyncTaskResult::Ok
                });
                match result {
                    Err(err) => {println!("{}", err)}
                    _ => {}
                }
                count += 1;
                thread::sleep(Duration::new(2, 0));
            }

            rt.stop();
        }
        Err(_msg) => { panic!("{_msg}"); }
    }

}
