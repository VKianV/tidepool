#![feature(mpmc_channel)]
use std::{thread, sync::{mpmc}};

type Job = Box<dyn FnOnce() + Send + 'static>;

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
        let job = Box::new(f);

        self.sender.send(job).expect("sending job to worker failed");
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: mpmc::Receiver<Job> ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver
                    .recv()
                    .expect("failed to receive the job");

                println!("Worker {} got a job; executing.", id);

                job();
            }
        });

        Self { id, thread }
    }
}
