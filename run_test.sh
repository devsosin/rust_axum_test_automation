#!/bin/bash

# Enable script to stop on errors
set -e

# Step 1: Start the test database using Docker Compose
echo "Starting test database with Docker Compose..."
docker-compose up --build -d test_db

# Step 2: Run the tests with Docker Compose and save output to log
echo "Running tests with Docker Compose..."
docker-compose run --build --rm test_runner | tee raw_test_results.log

# Step 3: Analyze test results for coverage and extract the specific failures section
echo "Analyzing test results and extracting failures section..."

# Read the raw log file into an array
logContent=()
while IFS= read -r line; do
    logContent+=("$line")
done < raw_test_results.log

# Initialize log output
logOutput=()

# Extract the line with total tests summary
totalTestsLine=$(printf "%s\n" "${logContent[@]}" | grep "test result:")

# Extract total, passed, failed, ignored, and filtered out test counts
if [[ $totalTestsLine =~ ([0-9]+)\ passed;\ ([0-9]+)\ failed;\ ([0-9]+)\ ignored;\ ([0-9]+)\ measured;\ ([0-9]+)\ filtered\ out ]]; then
    passedTests=${BASH_REMATCH[1]}
    failedTests=${BASH_REMATCH[2]}
    ignoredTests=${BASH_REMATCH[3]}
    totalTests=$((passedTests + failedTests + ignoredTests))

    # Calculate coverage percentage
    if ((totalTests > 0)); then
        coverage=$(awk "BEGIN {print ($passedTests / $totalTests) * 100}")
        coverageOutput="Coverage: $(printf "%.2f" $coverage)% ($passedTests passed out of $totalTests)"
        echo "$coverageOutput"
        logOutput+=("$coverageOutput")
    else
        echo "No tests were run."
        logOutput+=("No tests were run.")
    fi

    # Extract the content between the first and last 'failures:' markers, excluding the last one
    firstFailuresIndex=$(printf "%s\n" "${logContent[@]}" | grep -n "^failures:$" | head -n 1 | cut -d: -f1)
    lastFailuresIndex=$(printf "%s\n" "${logContent[@]}" | grep -n "^failures:$" | tail -n 1 | cut -d: -f1)

    if [[ -n $firstFailuresIndex && -n $lastFailuresIndex && $firstFailuresIndex -ne $lastFailuresIndex ]]; then
        # Capture lines between the first 'failures:' and the last 'failures:', excluding the last 'failures:' line
        failuresSection=("${logContent[@]:firstFailuresIndex:lastFailuresIndex-firstFailuresIndex-2}")

        echo "Failures Section:"
        logOutput+=("")
        logOutput+=("Failures Section:")

        for line in "${failuresSection[@]}"; do
            # Replace ---- (.*) stdout ---- with Failed: .*
            if [[ $line =~ ----\ (.*)\ stdout\ ---- ]]; then
                formattedLine="Failed: ${BASH_REMATCH[1]}"
                echo "$formattedLine"
                logOutput+=("$formattedLine")
            # Remove lines with the specific RUST_BACKTRACE note
            elif [[ $line =~ note:\ run\ with\ .* ]]; then
                # Skip this line
                continue
            # Handle lines with panicked at
            elif [[ $line =~ panicked\ at\ (.+):([0-9]+:[0-9]+): ]]; then
                formattedLine="Location: ${BASH_REMATCH[1]}:${BASH_REMATCH[2]}"
                echo "$formattedLine"
                logOutput+=("$formattedLine")
            else
                # Output lines as they are if no specific formatting is needed
                echo "$line"
                logOutput+=("$line")
            fi
        done
    else
        echo "No specific failures section found between the markers."
        logOutput+=("No specific failures section found between the markers.")
    fi
else
    echo "Could not parse test summary."
    logOutput+=("Could not parse test summary.")
fi

# Save the filtered failures output to a summary log file
printf "%s\n" "${logOutput[@]}" > test_results_summary.log

# Step 4: Delete the raw test results log as it's no longer needed
rm -f raw_test_results.log
echo "Deleted raw_test_results.log after processing."

# Step 5: Clean up the environment by shutting down Docker Compose services
echo "Cleaning up the test environment..."
docker-compose down

# Notify the user of test completion and log locations
echo "Tests completed. Failures section is saved in test_results_summary.log."
