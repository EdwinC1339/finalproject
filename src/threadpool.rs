use std::{
    cmp::Ordering, collections::BinaryHeap, sync::mpsc, thread
};

// Worker

#[derive(Debug)]
struct Worker<T, U> {
    jobs: u32,
    tx: mpsc::Sender<(u32, T)>,
    rx: mpsc::Receiver<(u32, U)>,
    join_handle: thread::JoinHandle<()>,
}

impl<T, U> Ord for Worker<T, U> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.jobs.cmp(&self.jobs) // Comparing other with this to get min heap behavior
    }
}

impl<T, U> PartialOrd for Worker<T, U> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, U> PartialEq for Worker<T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.jobs == other.jobs
    }
}

impl<T, U> Eq for Worker<T, U> {}

// Pool

#[derive(Debug)]
pub struct ThreadPool<T, U> 
    where
            T: Send,
            U: Send
{
    workers: BinaryHeap<Worker<T, U>>,
    key_track: u32
}

impl<T, U> ThreadPool<T, U> 
    where
        T: Send + 'static,
        U: Send + 'static,
    {
    pub fn new (
        n_workers: usize,
        job: impl Fn(T) -> U + Send + Clone + 'static,
    ) -> Self {
        assert!(n_workers > 0);
        let mut workers = BinaryHeap::new();
        for _ in 0..n_workers {
            workers.push(Self::create_worker(job.clone()))
        }
        Self { workers, key_track: 0 }
    }

    fn create_worker<'a>(job: impl Fn(T) -> U + Send + 'static) -> Worker<T, U> {
        let (my_sender, their_receiver) = mpsc::channel::<(u32, T)>();
        let (their_sender, my_receiver) = mpsc::channel::<(u32, U)>();

        let join_handle = thread::spawn(move || {
            for (key, workload) in their_receiver {
                let output = job(workload);
                their_sender.send((key, output)).expect("My sender dropped while thread exists");
            }
        });

        Worker { jobs: 0, tx: my_sender, rx: my_receiver, join_handle }
    }

    pub fn submit(&mut self, value: T) -> u32 {
        let mut selected_worker = self.workers.pop().expect("Zero workers");
        self.key_track += 1;
        selected_worker.jobs += 1;
        selected_worker.tx.send((self.key_track, value)).unwrap();
        self.workers.push(selected_worker);
        self.key_track
    }

    pub fn poll(&mut self) -> Vec<(u32, U)> {
        let mut collection = vec![];
        let mut worker_stack = vec![];
        while let Some(mut worker) = self.workers.pop() {
            while let Ok(out) = worker.rx.try_recv() {
                worker.jobs -= 1;
                collection.push(out);
            }

            worker_stack.push(worker);
        }

        self.workers.extend(worker_stack);

        collection
    }

    pub fn join(mut self) {
        while let Some(worker) = self.workers.pop() {
            drop(worker.tx);
            drop(worker.rx);
            worker.join_handle.join().expect("Couldn't join worker thread");
        }
    }
}