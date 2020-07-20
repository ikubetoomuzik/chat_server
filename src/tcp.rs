// Need to create and manage a tcp server instance then make a wrapper.
//

use std::net::{SocketAddr,TcpListener};

mod thread_pool;

#[derive(Debug)]
pub struct TcpServer {
    pub listener: Option<TcpListener>,
    // pub threads: thread_pool::ThreadPool,
}

impl TcpServer {
    pub fn new(size: usize) -> TcpServer {
        TcpServer {
            listener: None,
            // threads: thread_pool::ThreadPool::new(size),
        }
    }

    pub fn bind(&mut self, port: u16) {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        self.listener = Some(TcpListener::bind(&addr).unwrap());
    }
}
