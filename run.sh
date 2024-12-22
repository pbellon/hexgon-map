#!/bin/bash

script_dir=$(dirname "$0")

# Define the script usage
usage() {
  echo "Usage: $0 {test|up|stress-test}"
  exit 1
}

# Check if an argument is provided
if [ -z "$1" ]; then
  usage
fi

# Handle the commands 
case "$1" in
  test)
    echo "Running tests..."
    cd "$script_dir/server" || exit
    docker-compose -f docker-compose.override.yml up --build --remove-orphans
    cd - || exit
    ;;
  up)
    echo "Starting the application..."
    cd "$script_dir/server" || exit
    docker-compose -f docker-compose.yml up --build --remove-orphans
    cd - || exit
    ;;
  stress-test)
    echo "Starting K6 stress test suite"
    cd "$script_dir/stress-test" || exit
    K6_WEB_DASHBOARD=true k6 run test.js
    cd - || exit
    ;;
  *)
    usage
    ;;
esac
