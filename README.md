My implementation of the [final project](https://doc.rust-lang.org/stable/book/ch21-00-final-project-a-web-server.html) from The Rust Programming Language book.

# Notable changes

Compared to the book's version, my project's thread pool implementation is far more efficient. I chose to implement mine such that all threads are created at the start of the program, and the server responds to requests by sending the requests to pre-existing worker threads.

This approach minimizes overhead because it does not require additional threads to be created during the server's lifetime. The original implementation would create a thread every single time a request was made, which in the case of small requests meant more time was spent by the OS creating threads than actually doing useful work.

My version also implemented some HTTP parsing functionality that the book's version does not.
