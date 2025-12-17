//! Riotpool: A simple thread pool implementation
//!
//! This crate provides a basic thread pool for executing closures concurrently.
//! It uses a channel to distribute jobs to a fixed number of worker threads.
//!
//! Graceful shutdown is handled via `Drop`: when the pool is dropped,
//! the sender is closed, causing workers to exit after finishing their current job.

use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A thread pool that executes jobs concurrently across a fixed number of worker threads.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// `size` is the number of worker threads.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let pool = riotpool::ThreadPool::new(8);
    /// ```
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 1..=size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Execute a closure on one of the worker threads.
    ///
    /// The closure must be `Send + 'static` and will be executed exactly once.
    ///
    /// # Panics
    ///
    /// Panics if the internal channel is closed (i.e., the pool has already been dropped).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// pool.execute(|| {
    ///     println!("Hello from a worker thread!");
    /// });
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .as_ref()
            .expect("sender is not available anymore")
            .send(job)
            .expect("sending job to worker failed");
    }
}

/// Gracefully shuts down all workers when the pool is dropped.
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Close the channel to signal workers to shut down
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().expect("failed to join worker");
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker thread that continuously receives jobs from the shared receiver.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("failed to acquire the lock")
                .recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}