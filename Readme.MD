# Running the application
1. Install the latest Rust build
2. Build the current source code by running `cargo build`
3. Make sure the database server is running and .env is pointing towards the proper server
4. Serve the service by running `cargo run`

# Migrations
1. Install sqlx-cli by running `cargo install sqlx-cli`
2. Run the command `sqlx migrate add initial`
3. There is `RUN_MIGRATIONS` and `SEED` boolean variable available in the .env file. Set these to `true` or `false` depending on use case.
