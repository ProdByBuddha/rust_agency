#!/bin/bash
# agent_ci.sh - The Organism's Internal Immune Response
# 
# Designed to be invoked by the Agent itself after self-mutation.
# Returns structured, actionable feedback.

set -o pipefail

# 1. ENVIRONMENT ISOLATION
# Force "Dry Run" mode for all dangerous subsystems
export AGENCY_USE_REMOTE_MEMORY=0
# Allow download strategy for correct version
export ORT_STRATEGY=download

# Copy dylib to root for tests that don't respect ORT_DYLIB_PATH
# cp artifacts/bin/libonnxruntime.dylib . # Disabled: Let ort fetch correct version

cleanup() {
    rm -f libonnxruntime.dylib
}
trap cleanup EXIT

LOG_FILE="logs/agent_ci.log"
mkdir -p logs

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') $1" | tee -a "$LOG_FILE"
}

fail() {
    echo "{"status": "failure", "stage": "$1", "error": "$2"}"
    log "❌ FAILED: $1 - $2"
    exit 1
}

success() {
    echo "{"status": "success", "message": "All systems verified."}"
    log "✅ SUCCESS: Organism integrity confirmed."
    exit 0
}

# 2. SYNTAX CHECK (The Reflex)
log "Running syntax check..."
if ! cargo check --bin rust_agency > /dev/null 2>> "$LOG_FILE"; then
    fail "syntax" "Compilation failed. See logs/agent_ci.log for compiler errors."
fi

# 3. UNIT VERIFICATION (The Logic)
log "Running unit logic verification..."
if ! cargo test --lib > /dev/null 2>> "$LOG_FILE"; then
    fail "logic" "Unit tests failed. Logic integrity compromised."
fi

# 4. ARCHITECTURE CHECK (The Skeleton)
log "Running architecture verification..."
if ! cargo test --test architecture > /dev/null 2>> "$LOG_FILE"; then
    fail "structure" "Architecture tests failed. Structural rules violated."
fi

# 5. INTEGRATION CHECK (The Systems)
log "Running integration systems verification..."
# We skip heavy load tests for CI speed, focusing on correctness
if ! cargo test --test comprehensive_features > /dev/null 2>> "$LOG_FILE"; then
    fail "integration" "Comprehensive feature tests failed. Organ systems malfunction."
fi

success
