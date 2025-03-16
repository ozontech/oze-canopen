use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// A struct that holds a collection of `JoinHandle` tasks and a flag to control their termination.
pub struct JoinHandles {
    /// A vector of `JoinHandle` representing asynchronous tasks.
    pub(crate) handles: Vec<JoinHandle<()>>,
    /// A shared flag to signal the tasks to terminate.
    pub(crate) close_flag: Arc<Mutex<bool>>,
}

impl JoinHandles {
    /// Initiates the termination of all tasks and awaits their completion.
    ///
    /// This function sets the `close_flag` to `true`, signaling all tasks to stop.
    /// It then waits for each task to complete.
    pub async fn close_and_join(&mut self) {
        // Set the close flag to true.
        self.close_flag.lock().await.clone_from(&true);
        // Await the completion of each task.
        for i in &mut self.handles {
            i.await.unwrap();
        }

        self.handles.clear();
    }
}
