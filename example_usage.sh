#!/bin/bash

# Example usage of the secrets manager
# This script demonstrates how to use the secrets manager CLI

echo "🔐 Secrets Manager Example Usage"
echo "================================="
echo

# Build the project
echo "Building the project..."
cargo build --release
echo

# Create a new project
echo "Creating a new project 'example-api'..."
echo "Note: You'll be prompted for a master password"
cargo run -- create example-api
echo

# Add some secrets
echo "Adding secrets to the project..."
cargo run -- add example-api API_KEY "sk-1234567890abcdef"
cargo run -- add example-api DATABASE_URL "postgres://user:password@localhost:5432/mydb"
cargo run -- add example-api REDIS_URL "redis://localhost:6379"
cargo run -- add example-api JWT_SECRET "super-secret-jwt-key"
echo

# List all projects
echo "Listing all projects..."
cargo run -- list
echo

# Show project details
echo "Showing project details..."
cargo run -- show example-api
echo

# Export to different formats
echo "Exporting to shell format..."
cargo run -- export example-api --format shell
echo

echo "Exporting to .env file..."
cargo run -- export example-api --format env --output example.env
echo "Created example.env file"
echo

echo "Exporting to JSON format..."
cargo run -- export example-api --format json
echo

# Clean up
echo "Cleaning up..."
rm -f example.env
cargo run -- delete example-api
echo "Example completed!"
