use std::{
    sync::{
        Arc,
        mpsc::sync_channel, Mutex,
    },
    task::{Context, Poll},
    thread::{self, JoinHandle},
    time::Duration,
};
use std::sync::mpsc::TryRecvError;

use futures::task::waker_ref;

use crate::async_runtime::async_task::{AsyncTask, AsyncTaskResult};
pub use crate::async_runtime::spawner::{AsyncReceiver, AsyncSender, Spawner};

pub(crate) mod spawner;
pub(crate) mod async_task;

pub(crate) const MAX_QUEUED_TASKS: usize = 10_000;

#[derive(Debug)]
pub(crate) struct AsyncRuntime {
    status: RuntimeStatus,
    worker: Option<Worker>,
    spawner: Option<Spawner>,
}

#[derive(Debug)]
enum RuntimeStatus { Stopped, Running }


#[derive(Debug)]
pub(crate) struct Worker {
    status: Arc<Mutex<WorkerStatus>>,
    _join_handle: JoinHandle<()>,
}

#[derive(Debug, PartialEq)]
enum WorkerStatus { Stopped, Running, Stopping }

impl AsyncRuntime {
    pub(crate) fn new() -> Result<Self, String> {
        let new_runtime = Self {
            status: RuntimeStatus::Stopped,
            worker: None,
            spawner: None,
        };

        Ok(new_runtime)
    }

    pub(crate) fn start(&mut self) -> Result<(), String> {
        self.start_worker_thread()?;
        Ok(())
    }

    fn start_worker_thread(&mut self) -> Result<(), String> {
        let (snd, rcv): (AsyncSender<AsyncTask>, AsyncReceiver<AsyncTask>) = sync_channel(MAX_QUEUED_TASKS);

        let status_mutex = Arc::new(Mutex::new(WorkerStatus::Running));
        let worker: Worker = Worker {
            status: status_mutex.clone(),
            _join_handle: AsyncRuntime::start_thread(status_mutex, rcv),
        };

        self.worker = Some(worker);
        self.spawner = Some(Spawner::new(String::from("Runtime"), snd));

        Ok(())
    }

    fn start_thread(status: Arc<Mutex<WorkerStatus>>, receiver: AsyncReceiver<AsyncTask>) -> JoinHandle<()> {
        thread::spawn(move || {
            let local_receiver = receiver;
            loop {
                let mut local_status = status.lock().unwrap();

                match *local_status {
                    WorkerStatus::Stopped => {
                        break;
                    }
                    WorkerStatus::Running => {
                        match &local_receiver.try_recv() {
                            Ok(async_task) => {
                                // Took task
                                let mut future_slot = async_task.future.lock().unwrap();
                                if let Some(mut future) = future_slot.take() {
                                    let waker = waker_ref(&async_task);
                                    let context = &mut Context::from_waker(&waker);
                                    if let Poll::Pending = future.as_mut().poll(context) {
                                        *future_slot = Some(future);
                                    }
                                }
                            }
                            Err(err) => {
                                if let TryRecvError::Disconnected = err {
                                    //todo: Handle disconnection
                                }
                            }
                        }
                    }
                    WorkerStatus::Stopping => {
                        println!("Stopping!");
                        *local_status = WorkerStatus::Stopped;
                    }
                }
                thread::sleep(Duration::new(0, 500000));
            }
        })
    }

    pub(crate) fn get_spawner(&self) -> Result<Spawner, String> {
        match &self.spawner {
            None => Err(String::from("No spawner!")),
            Some(spawner) => Ok(spawner.clone())
        }
    }

    pub(crate) fn stop(&mut self) {
        if let Some(worker) = &self.worker {
            let mut status = worker.status.lock().unwrap();
            *status = WorkerStatus::Stopping;
        }
    }
}