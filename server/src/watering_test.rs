use std::sync::Arc;

use tokio::sync::{
    oneshot::{channel, Receiver, Sender},
    Mutex,
};

pub type PlantId = String;
pub type TaskError = ();

struct Task<T, R> {
    value: T,
    ack: Sender<R>,
}

impl<T, R> Task<T, R> {
    pub fn new(value: T) -> (Self, Receiver<R>) {
        let (ack, response) = channel();
        (Self { value, ack }, response)
    }
}

#[derive(Clone)]
pub struct PendingWateringTest {
    // inner: Arc<Mutex<Option<(Receiver<usize>, <()>)>>>,
    inner: Arc<Mutex<Option<Task<PlantId, Result<(), TaskError>>>>>,
}

impl PendingWateringTest {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_pending_job(&self, plant_id: PlantId) -> (Receiver<Result<(), TaskError>>) {
        let (task, response) = Task::new(plant_id);
        let mut inner = self.inner.lock().await;
        let _ = inner.insert(task);
        response
    }

    pub async fn pop_pending_task(&self) -> Option<Task<PlantId, Result<(), TaskError>>> {
        let mut inner = self.inner.lock().await;
        inner.take()
    }
}
