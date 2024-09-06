#!/bin/bash
set -e

# Step 1: Start the test database using Docker Compose
echo "Starting test database with Docker Compose..."
docker-compose up --build -d test_db

# Step 2: Run the tests with Docker Compose and save output to log
echo "Running tests with Docker Compose..."
docker-compose run --build --rm test_runner | tee raw_test_results.log

# Step 3: Analyze test results for coverage and extract the specific failures section
echo "Analyzing test results and extracting failures section..."

# Read the raw log file
logContent=$(cat raw_test_results.log)

# Initialize log output
logOutput=""

# 파일명을 변수로 선언
logFile="test_results_summary.log"


#!/bin/bash
set -e

# Step 2: Extract test result summary
totalTestsLine=$(echo "$logContent" | grep "test result:")

# Extract total, passed, and failed test counts using a simpler approach
passedTests=$(echo "$totalTestsLine" | grep -o '[0-9]\+ passed' | grep -o '[0-9]\+')
failedTests=$(echo "$totalTestsLine" | grep -o '[0-9]\+ failed' | grep -o '[0-9]\+')

if [ -n "$passedTests" ] && [ -n "$failedTests" ]; then
    totalTests=$((passedTests + failedTests))

    # Calculate coverage percentage
    if [ "$totalTests" -gt 0 ]; then
        coverage=$(echo "scale=2; ($passedTests / $totalTests) * 100" | bc)
        echo "Coverage: $coverage% ($passedTests passed out of $totalTests)\n" > "$logFile"
    else
        echo "No tests were run." > "$logFile"
    fi
else
    echo "Could not parse test summary." > "$logFile"
fi

echo "Failures Section:" >> "$logFile"

firstIdx=$(grep -n "^failures:$" <<< "$logContent" | head -1 | cut -d: -f1)

if [ -z "$firstIdx" ]; then
    echo "No 'failures' found." >> "$logFile"
else 
    # 첫 번째 "failures:" 이후 로그 잘라내기
    tailContent=$(sed -n "${firstIdx},\$p" <<< "$logContent")

    # 잘라낸 로그에서 다음 "failures:" 줄 번호 찾기
    secondIdx=$(grep -n "^failures:$" <<< "$tailContent" | head -2 | tail -1 | cut -d: -f1)

    # 첫 번째와 두 번째 "failures:" 사이의 내용 추출
    finalContent=$(sed -n "2,$((secondIdx-1))p" <<< "$tailContent")

    while IFS= read -r line; do
        # '---- 내용 stdout ----' 패턴을 'Failure: 내용'으로 변경
        if [[ "$line" =~ ----\ (.*)\ stdout\ ---- ]]; then
            content="${BASH_REMATCH[1]}"
            echo "Failure: $content" >> "$logFile"
            continue
        fi

        # 'note: run with `RUST_BACKTRACE=1`'가 포함된 줄은 출력하지 않음
        if [[ "$line" == *"note: run with `RUST_BACKTRACE=1`"* ]]; then
            continue
        fi

        # 'panicked at'를 포함한 줄에서 메시지 추출
        if [[ "$line" == *"thread"* && "$line" == *"panicked at"* ]]; then
            # Extract the part after 'panicked at'
            echo "$line" | awk -F 'panicked at ' '{print "Location: " $2}' | sed 's/:$//' >> "$logFile"
            continue
        fi

        echo "$line" >> "$logFile"

    done <<< "$finalContent"
fi

# Save the filtered failures output to a summary log file
# printf "%s\n" "$logOutput" >> test_results_summary.log

# Step 4: Delete the raw test results log as it's no longer needed
rm -f raw_test_results.log
echo "Deleted raw_test_results.log after processing."

# Step 5: Clean up the environment by shutting down Docker Compose services
echo "Cleaning up the test environment..."
docker-compose down

# Notify the user of test completion and log locations
echo "Tests completed. Failures section is saved in test_results_summary.log."
