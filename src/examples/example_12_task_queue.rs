// Example 12: Task Queue Implementation
//
// This example demonstrates how to implement a background task queue system
// using Rust's async capabilities and channels. It shows how to create a
// system that can process tasks asynchronously in the background while
// allowing the main application to continue running.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

// Type alias for task functions
// This represents a task that can be executed asynchronously
// Tasks are boxed functions that return a Result
type Task = Box<dyn Fn() -> Result<String, String> + Send + 'static>;

// Enum: TaskPriority
//
// This enum defines different priority levels for tasks in our queue.
// Higher priority tasks will be processed before lower priority ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

// Struct: TaskItem
//
// This struct represents a single task item in the queue.
// It contains the task function itself along with metadata
// like priority and a unique identifier.
pub struct TaskItem {
    id: u64,
    priority: TaskPriority,
    task: Task,
    description: String,
}

impl std::fmt::Debug for TaskItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskItem")
            .field("id", &self.id)
            .field("priority", &self.priority)
            .field("description", &self.description)
            .field("task", &"<function>")
            .finish()
    }
}

impl TaskItem {
    // Function: new
    //
    // Creates a new task item with the given parameters.
    //
    // Arguments:
    //     id: A unique identifier for this task
    //     priority: The priority level of this task
    //     task: The actual function to execute
    //     description: A human-readable description of the task
    //
    // Returns:
    //     A new TaskItem instance
    pub fn new(id: u64, priority: TaskPriority, task: Task, description: String) -> Self {
        Self {
            id,
            priority,
            task,
            description,
        }
    }

    // Function: execute
    //
    // Executes the task function and returns the result.
    // This consumes the TaskItem since tasks should only be executed once.
    //
    // Returns:
    //     Result containing the task output or an error message
    pub fn execute(self) -> Result<String, String> {
        info!("Executing task {}: {}", self.id, self.description);
        (self.task)()
    }
}

// Struct: TaskQueue
//
// This struct implements a priority-based task queue that can process
// tasks asynchronously in the background. It uses tokio channels for
// communication between the main thread and worker threads.
pub struct TaskQueue {
    sender: mpsc::UnboundedSender<TaskItem>,
    shutdown_notify: Arc<Notify>,
    next_task_id: Arc<Mutex<u64>>,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    // Function: new
    //
    // Creates a new task queue and starts the background worker.
    // The worker will continuously poll for new tasks and execute them
    // based on their priority.
    //
    // Returns:
    //     A new TaskQueue instance
    pub fn new() -> Self {
        // Create an unbounded channel for task communication
        // Unbounded channels allow unlimited queueing of tasks
        let (sender, receiver) = mpsc::unbounded_channel::<TaskItem>();

        // Create a notification mechanism for graceful shutdown
        let shutdown_notify = Arc::new(Notify::new());
        let shutdown_notify_worker = shutdown_notify.clone();

        // Initialize the task ID counter
        let next_task_id = Arc::new(Mutex::new(1u64));

        // Spawn the background worker task
        // This task will run continuously until shutdown is requested
        tokio::spawn(async move {
            Self::worker_loop(receiver, shutdown_notify_worker).await;
        });

        info!("Task queue initialized and worker started");

        Self {
            sender,
            shutdown_notify,
            next_task_id,
        }
    }

    // Function: add_task
    //
    // Adds a new task to the queue with the specified priority.
    // The task will be executed by the background worker when its turn comes.
    //
    // Arguments:
    //     priority: The priority level for this task
    //     task: The function to execute
    //     description: A description of what this task does
    //
    // Returns:
    //     Result indicating success or failure to queue the task
    pub async fn add_task<F>(
        &self,
        priority: TaskPriority,
        task: F,
        description: String,
    ) -> Result<u64, String>
    where
        F: Fn() -> Result<String, String> + Send + 'static,
    {
        // Generate a unique ID for this task
        let mut next_id = self.next_task_id.lock().await;
        let task_id = *next_id;
        *next_id += 1;
        drop(next_id); // Release the lock early

        // Create the task item
        let task_item = TaskItem::new(task_id, priority, Box::new(task), description.clone());

        // Send the task to the worker
        // If the channel is closed, the worker has shut down
        match self.sender.send(task_item) {
            Ok(_) => {
                info!(
                    "Queued task {}: {} (priority: {:?})",
                    task_id, description, priority
                );
                Ok(task_id)
            }
            Err(_) => {
                error!("Failed to queue task: worker has shut down");
                Err("Task queue is shut down".to_string())
            }
        }
    }

    // Function: shutdown
    //
    // Initiates a graceful shutdown of the task queue.
    // This will notify the worker to stop processing new tasks
    // and complete any currently running tasks.
    pub fn shutdown(&self) {
        info!("Initiating task queue shutdown");
        self.shutdown_notify.notify_one();
    }

    // Function: worker_loop
    //
    // This is the main worker loop that runs in the background.
    // It continuously receives tasks from the channel and executes them
    // in priority order. The loop will exit when shutdown is requested.
    //
    // Arguments:
    //     receiver: The channel receiver for incoming tasks
    //     shutdown_notify: Notification mechanism for shutdown
    async fn worker_loop(
        mut receiver: mpsc::UnboundedReceiver<TaskItem>,
        shutdown_notify: Arc<Notify>,
    ) {
        // Use a priority queue to ensure high-priority tasks are executed first
        let mut task_buffer: VecDeque<TaskItem> = VecDeque::new();

        info!("Task queue worker started");

        loop {
            // Use tokio::select! to handle both incoming tasks and shutdown signals
            tokio::select! {
                // Handle incoming tasks
                task_option = receiver.recv() => {
                    match task_option {
                        Some(task) => {
                            // Insert the task in priority order
                            Self::insert_task_by_priority(&mut task_buffer, task);

                            // Process all available tasks in the buffer
                            Self::process_task_buffer(&mut task_buffer).await;
                        }
                        None => {
                            // Channel closed, no more tasks will arrive
                            warn!("Task channel closed, worker shutting down");
                            break;
                        }
                    }
                }

                // Handle shutdown signal
                _ = shutdown_notify.notified() => {
                    info!("Shutdown signal received, processing remaining tasks");

                    // Process any remaining tasks in the buffer
                    Self::process_task_buffer(&mut task_buffer).await;

                    // Process any remaining tasks in the channel
                    while let Ok(task) = receiver.try_recv() {
                        Self::insert_task_by_priority(&mut task_buffer, task);
                    }
                    Self::process_task_buffer(&mut task_buffer).await;

                    info!("Worker shutdown complete");
                    break;
                }
            }
        }
    }

    // Function: insert_task_by_priority
    //
    // Inserts a task into the buffer maintaining priority order.
    // Higher priority tasks are placed at the front of the queue.
    //
    // Arguments:
    //     buffer: The task buffer to insert into
    //     task: The task to insert
    fn insert_task_by_priority(buffer: &mut VecDeque<TaskItem>, task: TaskItem) {
        // Find the correct position to insert the task based on priority
        let insert_position = buffer
            .iter()
            .position(|existing_task| existing_task.priority < task.priority)
            .unwrap_or(buffer.len());

        buffer.insert(insert_position, task);
    }

    // Function: process_task_buffer
    //
    // Processes all tasks currently in the buffer.
    // Tasks are executed in priority order (highest priority first).
    //
    // Arguments:
    //     buffer: The task buffer to process
    async fn process_task_buffer(buffer: &mut VecDeque<TaskItem>) {
        while let Some(task) = buffer.pop_front() {
            let task_id = task.id;

            // Execute the task and handle the result
            match task.execute() {
                Ok(result) => {
                    info!("Task {} completed successfully: {}", task_id, result);
                }
                Err(error) => {
                    error!("Task {} failed: {}", task_id, error);
                }
            }

            // Add a small delay between tasks to prevent overwhelming the system
            // In a real-world scenario, this might be configurable
            sleep(Duration::from_millis(10)).await;
        }
    }
}

// Function: create_sample_task
//
// Creates a sample task function for demonstration purposes.
// This function simulates some work by sleeping and then returning a result.
//
// Arguments:
//     task_name: A name for this task
//     work_duration_ms: How long the task should simulate working
//     should_fail: Whether this task should simulate a failure
//
// Returns:
//     A boxed task function that can be added to the queue
fn create_sample_task(
    task_name: String,
    work_duration_ms: u64,
    should_fail: bool,
) -> Box<dyn Fn() -> Result<String, String> + Send + 'static> {
    Box::new(move || {
        // Simulate some work
        std::thread::sleep(Duration::from_millis(work_duration_ms));

        if should_fail {
            Err(format!("Task '{}' failed as requested", task_name))
        } else {
            Ok(format!(
                "Task '{}' completed after {}ms",
                task_name, work_duration_ms
            ))
        }
    })
}

// Function: main
//
// This is the entry point of the program.
// It demonstrates the task queue system by creating various tasks
// with different priorities and execution characteristics.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracing subscriber for logging
    // This will show us what's happening with our tasks
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Task Queue Example");

    // Create a new task queue
    let task_queue = TaskQueue::new();

    // Add various tasks with different priorities
    info!("Adding tasks to the queue...");

    // Add a high-priority task
    task_queue
        .add_task(
            TaskPriority::High,
            create_sample_task("High Priority Task".to_string(), 100, false),
            "Critical system maintenance".to_string(),
        )
        .await?;

    // Add some normal priority tasks
    for i in 1..=3 {
        task_queue
            .add_task(
                TaskPriority::Normal,
                create_sample_task(format!("Normal Task {}", i), 50, false),
                format!("Regular processing task {}", i),
            )
            .await?;
    }

    // Add a low priority task
    task_queue
        .add_task(
            TaskPriority::Low,
            create_sample_task("Low Priority Task".to_string(), 200, false),
            "Background cleanup".to_string(),
        )
        .await?;

    // Add a critical priority task (should be processed first)
    task_queue
        .add_task(
            TaskPriority::Critical,
            create_sample_task("Critical Task".to_string(), 75, false),
            "Emergency response".to_string(),
        )
        .await?;

    // Add a task that will fail
    task_queue
        .add_task(
            TaskPriority::Normal,
            create_sample_task("Failing Task".to_string(), 30, true),
            "Task that demonstrates error handling".to_string(),
        )
        .await?;

    info!("All tasks queued. Waiting for processing...");

    // Give the worker some time to process the tasks
    sleep(Duration::from_secs(2)).await;

    // Add more tasks after initial processing
    info!("Adding additional tasks...");

    for i in 4..=6 {
        task_queue
            .add_task(
                TaskPriority::Normal,
                create_sample_task(format!("Additional Task {}", i), 40, false),
                format!("Late-added task {}", i),
            )
            .await?;
    }

    // Wait a bit more for the additional tasks to process
    sleep(Duration::from_secs(1)).await;

    // Demonstrate graceful shutdown
    info!("Initiating graceful shutdown...");
    task_queue.shutdown();

    // Give the worker time to complete shutdown
    sleep(Duration::from_millis(500)).await;

    info!("Task Queue Example completed successfully");

    Ok(())
}
