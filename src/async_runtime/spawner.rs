use std::future::Future;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, SyncSender};

use futures::FutureExt;
use uuid::Uuid;

use crate::async_runtime::{AsyncTask, AsyncTaskResult};

pub type AsyncSender<T> = SyncSender<Arc<T>>;
pub type AsyncReceiver<T> = Receiver<Arc<T>>;

#[derive(Debug, Clone)]
pub struct Spawner {
    source_name: String,
    sender: AsyncSender<AsyncTask>,
}

impl Spawner {
    pub fn spawn_non_blocking(&self, future: impl Future<Output = AsyncTaskResult> + 'static + Send) -> Result<(), String> {
        let future_boxed = future.boxed();

        let task = Arc::new(AsyncTask {
            id: Uuid::new_v4(),
            name: format!("{} task", self.source_name),
            future: Mutex::new(Some(future_boxed)),
            sender_handle: self.clone_sender(),
        });

        self.send(task)
    }

    pub fn send(&self, task: Arc<AsyncTask>) -> Result<(), String> {
        match self.sender.send(task) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Error spawning task: {}", err.to_string())),
        }
    }

    pub fn clone_sender(&self) -> AsyncSender<AsyncTask> {
        self.sender.clone()
    }
    pub fn new(source_name: String, sender: AsyncSender<AsyncTask>) -> Self { Self { source_name, sender } }
}