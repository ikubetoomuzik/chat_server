// Need to create and manage a tcp server instance then make a wrapper.
//

use std::net::{TcpListener, TcpStream};
use std::thread;

struct ThreadPool(Vec<Worker>);

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        let mut threads = Vec::new();

        for i in 0..size {
            threads.push(Worker::new(i));
        }

        ThreadPool(threads)
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize) -> Worker {
        Worker {
            id,
            thread: thread::spawn(|| {}),
        }
    }
}

struct TcpServer {
    listener: TcpListener,
    threads: ThreadPool,
}

#[cfg(test)]
mod tests {
    #[test]
    fn worker_new() {
        assert_eq!(
            Worker::new(0),
            Worker {
                id: 0,
                thread: thread::spawn(|| {})
            }
        );
    }
}
