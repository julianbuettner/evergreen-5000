use std::sync::Arc;

use tokio::sync::{
    oneshot::{channel, Receiver, Sender},
    Mutex,
};

use crate::model::WateringJob;

pub struct Task<T> {
    value: T,
    ack: Sender<()>,
}

impl<T> Task<T> {
    pub fn new(value: T) -> (Self, Receiver<()>) {
        let (ack, response) = channel();
        (Self { value, ack }, response)
    }

    pub fn destruct_and_ack(self) -> T {
        self.ack.send(());
        self.value
    }
}

#[derive(Clone)]
pub struct PendingWateringTest {
    inner: Arc<Mutex<Option<Task<WateringJob>>>>,
}

impl PendingWateringTest {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_pending_job(&self, plant_id: WateringJob) -> (Receiver<()>) {
        let (task, response) = Task::new(plant_id);
        let mut inner = self.inner.lock().await;
        let _ = inner.insert(task);
        response
    }

    pub async fn pop_pending_task(&self) -> Option<Task<WateringJob>> {
        let mut inner = self.inner.lock().await;
        inner.take()
    }
}
