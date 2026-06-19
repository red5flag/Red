#!/bin/bash
# Run cargo leptos watch with SSR/CSR logging enabled
# Captures all errors into errors.txt

export RUST_LOG=debug,leptos=trace,cargo_leptos=debug

ERRORS_FILE="errors.txt"

# Clear errors.txt at start
> "$ERRORS_FILE"

echo "Starting Channel Manager with full logging..."
echo "Server will be available at: http://10.241.68.184:3000"
echo "Logs will show both SSR and CSR build processes"
echo "Errors will be saved to $ERRORS_FILE"
echo ""

# Run cargo leptos watch, tee all output to stdout and capture errors
cargo leptos watch 2>&1 | tee >(grep -iE "^error|^warning:|could not compile|build failed" >> "$ERRORS_FILE")
