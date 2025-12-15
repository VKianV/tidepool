#![feature(mpmc_channel)]
use std::{thread, sync::{mpmc, Arc, Mutex}};

struct Job;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpmc::Sender<Job>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpmc::channel();


        for id in 1..=size {
            workers.push(Worker::new(id, receiver.clone()));
        }

        Self { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: mpmc::Receiver<Job> ) -> Self {
        let thread = thread::spawn(|| {
            receiver;
        });

        Self { id, thread }
    }
}
