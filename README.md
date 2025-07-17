# rust simple logger

A minimal implementation of a Rust logger.

https://crates.io/crates/rs_logger

## Features

- Logs are printed to stderr by default, but can be configured to any object that implements `std::io::Write`
- Supports inserting custom information in the middle of logs
- Supports logging without initializing the logging framework (using log_print!)
- Supports configuring whether log levels are displayed in color through features

![demo](./assets/readme.png)