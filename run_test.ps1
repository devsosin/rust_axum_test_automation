# Enable script to stop on errors
$ErrorActionPreference = "Stop"

# Step 1: Start the test database using Docker Compose
Write-Host "Starting test database with Docker Compose..."
docker-compose -f docker-compose.test.yml up --build -d db

# Step 2: Run the tests with Docker Compose and save output to log
Write-Host "Running tests with Docker Compose..."
docker-compose -f docker-compose.test.yml run --build --rm tester | Tee-Object -FilePath raw_test_results.log

# Step 3: Analyze test results for coverage and extract the specific failures section
Write-Host "Analyzing test results and extracting failures section..."

# Read the raw log file
$logContent = Get-Content raw_test_results.log

# Initialize log output
$logOutput = @()

# Extract the line with total tests summary
$totalTestsLine = ($logContent | Select-String -Pattern "test result:").Line

# Extract total, passed, failed, ignored, and filtered out test counts
if ($totalTestsLine -match "test result: .+? (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;") {
    $passedTests = [int]$matches[1]
    $failedTests = [int]$matches[2]
    $totalTests = $passedTests + $failedTests + [int]$matches[3]

    # Calculate coverage percentage
    if ($totalTests -gt 0) {
        $coverage = [math]::Round(($passedTests / $totalTests) * 100, 2)
        $coverageOutput = "Coverage: $coverage% ($passedTests passed out of $totalTests)`n"
        Write-Host $coverageOutput
        $logOutput += $coverageOutput
    } else {
        Write-Host "No tests were run.`n"
        $logOutput += "No tests were run.`n"
    }

    # Extract the content between the first and last 'failures:' markers, excluding the last one
    $firstFailuresIndex = ($logContent | Select-String -Pattern "^failures:$" | Select-Object -First 1).LineNumber
    $lastFailuresIndex = ($logContent | Select-String -Pattern "^failures:$" | Select-Object -Last 1).LineNumber

    if ($firstFailuresIndex -and $lastFailuresIndex -and $firstFailuresIndex -ne $lastFailuresIndex) {
        # Capture lines between the first 'failures:' and the last 'failures:', excluding the last 'failures:' line
        $failuresSection = $logContent[($firstFailuresIndex + 1)..($lastFailuresIndex - 2)]

        Write-Host "Failures Section:"
        $logOutput += "`nFailures Section:`n"

        # Write-Host $failuresSection -Join "`n"
        # $logOutput += $failuresSection

        foreach ($line in $failuresSection) {
            # Replace ---- (.*) stdout ---- with Failed: .*
            if ($line -match "---- (.*) stdout ----") {
                $formattedLine = "Failed: $($matches[1])"
                Write-Host $formattedLine
                $logOutput += $formattedLine
            }
            # Remove lines with the specific RUST_BACKTRACE note
            elseif ($line -match "note: run with .*") {
                # Skip this line
                continue
            }
            # Handle lines with panicked at
            elseif ($line -match "panicked at (.+):(\d+:\d+)") {
                $formattedLine = "Location: $($matches[1]):$($matches[2] -replace ':$', '')"
                Write-Host $formattedLine
                $logOutput += $formattedLine
            }
            else {
                # Output lines as they are if no specific formatting is needed
                Write-Host $line
                $logOutput += $line
            }
        }
    } else {
        Write-Host "No specific failures section found between the markers."
        $logOutput += "No specific failures section found between the markers."
    }
} else {
    Write-Host "`nCould not parse test summary."
    $logOutput += "`nCould not parse test summary."
}

# Save the filtered failures output to a summary log file
$logOutput | Out-File -FilePath test_results_summary.log

# Step 4: Delete the raw test results log as it's no longer needed
Remove-Item raw_test_results.log -ErrorAction SilentlyContinue
Write-Host "Deleted raw_test_results.log after processing."

# Step 5: Clean up the environment by shutting down Docker Compose services
Write-Host "Cleaning up the test environment..."
docker-compose -f docker-compose.test.yml down

# Notify the user of test completion and log locations
Write-Host "Tests completed. Failures section is saved in test_results_summary.log."
