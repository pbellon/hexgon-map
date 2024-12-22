#!/bin/bash

# Define the script usage
usage() {
  echo "Usage: $0 {test|up}"
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
    docker-compose -f docker-compose.override.yml up --build --remove-orphans
    ;;
  up)
    echo "Starting the application..."
    docker-compose -f docker-compose.yml up --build --remove-orphans
    ;;
  *)
    usage
    ;;
esac
