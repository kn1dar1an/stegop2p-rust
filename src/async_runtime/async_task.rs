use std::sync::{Arc, Mutex};

use futures::future::BoxFuture;
use futures::task::ArcWake;

use crate::async_runtime::spawner::AsyncSender;

type AsyncTaskFutureMutex<R> = Mutex<Option<BoxFuture<'static, R>>>;

pub struct AsyncTask{
    pub id: uuid::Uuid,
    pub name: String,
    pub future: AsyncTaskFutureMutex<AsyncTaskResult>,
    pub sender_handle: AsyncSender<AsyncTask>,
}

impl ArcWake for AsyncTask {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        //Clone to not consume the self arc reference
        let cloned_self = arc_self.clone();
        arc_self
            .sender_handle
            .send(cloned_self)
            .expect("Too many tasks queued!");
    }
}

pub trait TaskResult { }
impl TaskResult for String { }
impl TaskResult for () { }

pub enum AsyncTaskResult {
    Ok,
    Error(String)
}

#[derive(Debug)]
pub(crate) struct AsyncTaskError {
    pub(crate) message: String,
}

