# Enable script to stop on errors
$ErrorActionPreference = "Stop"

# Step 1: Start the test database using Docker Compose
Write-Host "Starting test database with Docker Compose..."
docker-compose up --build -d

# Step 2: Wait for the database to be ready
Write-Host "Waiting for the database to be ready..."
Start-Sleep -Seconds 6  # Adjust the sleep time as needed or improve with a health check loop

# Step 3: Run tests and capture output
Write-Host "Running tests..."
cargo test --no-fail-fast -- --test-threads=1 | Tee-Object -FilePath test_results.log

# Optional: Run test coverage using tarpaulin if installed
Write-Host "Running coverage analysis..."
cargo tarpaulin --out Html | Tee-Object -FilePath coverage.log

# Step 4: Shut down and clean up
Write-Host "Cleaning up test environment..."
docker-compose down

# Notify the user of test completion and log locations
Write-Host "Tests completed. Results are saved in test_results.log and coverage.log"
