// Need to create and manage a tcp server instance then make a wrapper.
//

use std::net::{TcpListener, TcpStream};

mod thread_pool;

struct TcpServer {
    listener: TcpListener,
    threads: thread_pool::ThreadPool,
}
