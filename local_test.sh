#!/bin/bash
set -e

# 변수 설정
DOCKER_COMPOSE_FILE="docker-compose.test.yml"
SERVICE_NAME="db"
TARGET=$1

# Health Check를 위한 최대 대기 시간 (초)
MAX_WAIT=60
SLEEP_INTERVAL=3

# Health Check 함수
check_health() {
  local status
  status=$(docker inspect --format='{{.State.Health.Status}}' "$(docker-compose -f "$DOCKER_COMPOSE_FILE" ps -q "$SERVICE_NAME")")
  echo "Current health status of $SERVICE_NAME: $status"
  if [ "$status" == "healthy" ]; then
    return 0
  elif [ "$status" == "unhealthy" ]; then
    echo "Service $SERVICE_NAME is unhealthy."
    return 1
  else
    return 1
  fi
}

# Step 1: Start the test database using Docker Compose
echo "Starting test database with Docker Compose..."
docker-compose -f "$DOCKER_COMPOSE_FILE" up --build -d "$SERVICE_NAME"

# Step 2: Wait for the database to become healthy
echo "Waiting for $SERVICE_NAME to become healthy..."
SECONDS_WAITED=0

while ! check_health; do
  if [ "$SECONDS_WAITED" -ge "$MAX_WAIT" ]; then
    echo "Error: $SERVICE_NAME did not become healthy within $MAX_WAIT seconds."
    echo "Shutting down Docker Compose services..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" down
    exit 1
  fi
  echo "Waiting for $SERVICE_NAME to be healthy... ($SECONDS_WAITED/$MAX_WAIT)"
  sleep "$SLEEP_INTERVAL"
  SECONDS_WAITED=$((SECONDS_WAITED + SLEEP_INTERVAL))
done

echo "$SERVICE_NAME is healthy. Proceeding with tests."

# Step 3: Set environment variable
export DATABASE_URL=postgres://test:test1234@localhost:5432/test_db

echo "초기화: pg_stat_statements_reset"
docker-compose -f "$DOCKER_COMPOSE_FILE" exec db psql -U test -d test_db -c "SELECT pg_stat_statements_reset();" > /dev/null

# Step 4: Run cargo test with the specified target
echo "Running tests for domain::$TARGET..."
cargo test "domain::$TARGET" -- --test-threads=1

# 테스트 실행 후 통계 수집
echo "통계 수집"
docker-compose -f "$DOCKER_COMPOSE_FILE" exec db psql -U test -d test_db -c "\
SELECT query, \
       calls, \
       total_exec_time AS total_time, \
       mean_exec_time AS mean_time, \
       rows \
FROM pg_stat_statements \
ORDER BY total_exec_time DESC \
LIMIT 10;" | tee -a query_results.log

# Step 5: Clean up the environment by shutting down Docker Compose services
echo "Cleaning up the test environment..."
docker-compose -f "$DOCKER_COMPOSE_FILE" down

echo "Test completed successfully."
