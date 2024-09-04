#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

# Step 1: Start the test database using Docker Compose
echo "Starting test database with Docker Compose..."
docker-compose up --build -d

# Give the database some time to initialize
echo "Waiting for the database to be ready..."
sleep 6  # Adjust sleep time as needed or use Docker health checks

# Step 2: Run tests and capture output
echo "Running tests..."
cargo test --no-fail-fast -- --test-threads=1 | tee test_results.log

# Optional: Run test coverage using tarpaulin if installed
echo "Running coverage analysis..."
cargo tarpaulin --out Html | tee coverage.log

# Step 3: Shut down and clean up
echo "Cleaning up test environment..."
docker-compose down

# Notify the user of test completion and log locations
echo "Tests completed. Results are saved in test_results.log and coverage.lo층dp