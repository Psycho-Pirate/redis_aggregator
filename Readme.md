# Redis Aggregator

## Overview
This project implements a real-time Redis aggregator that subscribes to multiple Redis channels, processes events in parallel, and publishes the processed data to a single output channel. It is built using Rust with Tokio for async processing and Redis for messaging.

## Features
- Subscribes to multiple Redis Pub/Sub channels (`inputA`, `inputB`, `inputC`)
- Uses Tokio for concurrent message handling
- Aggregates and processes incoming messages
- Publishes the processed results to a Redis output channel (`outputChannel`)
- Gracefully handles errors and shutdowns

## Prerequisites
- Install [Rust](https://www.rust-lang.org/tools/install)
- Install [Redis](https://redis.io/docs/getting-started/)
- Ensure Redis is running on `localhost:6379`

## Installation
1. Clone the repository:
   ```sh
   git clone https://github.com/your-repo/redis-aggregator.git
   cd redis-aggregator
   ```
2. Install dependencies:
   ```sh
   cargo build
   ```

## Running the Aggregator
Start the Redis server (if not already running):
```sh
redis-server
```

Run the aggregator:
```sh
cargo run
```

## Testing
To send test messages to the input channels, use the Redis CLI:
```sh
redis-cli PUBLISH inputA "hello"
redis-cli PUBLISH inputB "world"
redis-cli PUBLISH inputC "hello"
```
You should see aggregated results published to `outputChannel`.

To check published messages:
```sh
redis-cli SUBSCRIBE outputChannel
```

## Graceful Shutdown
To stop the application, press `Ctrl + C`. The aggregator will handle cleanup and shutdown properly.

## Logging
The application logs its status and errors using `env_logger`. To enable detailed logs, run:
```sh
RUST_LOG=info cargo run
```

## License
This project is licensed under the MIT License.

