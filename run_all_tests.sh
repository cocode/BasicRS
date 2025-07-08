#!/bin/bash

echo "Running unit tests..."
UNIT_RESULT=$(cargo test --lib -- --nocapture 2>&1)
UNIT_SUMMARY=$(echo "$UNIT_RESULT" | tail -1)

echo "Running BASIC test suite..."
BASIC_RESULT=$(cargo test --test run_tests -- --nocapture 2>&1)
BASIC_SUMMARY=$(echo "$BASIC_RESULT" | tail -1)

echo ""
echo "=== TEST SUMMARIES ==="
echo "Unit tests: $UNIT_SUMMARY"
echo "BASIC tests: $BASIC_SUMMARY" 