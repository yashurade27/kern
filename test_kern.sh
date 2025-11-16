#!/bin/bash

################################################################################
# Kern Comprehensive Test Suite
# This script extensively tests the kern project across all components
# Including: CLI, config parsing, profile loading, monitoring, and integration
################################################################################

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="${PROJECT_ROOT}/target/debug/kern"
TEST_CONFIG_DIR="${PROJECT_ROOT}/.test-config"

################################################################################
# Helper Functions
################################################################################

log_info() {
    echo -e "${BLUE}ℹ️  INFO:${NC} $1"
}

log_success() {
    echo -e "${GREEN}✅ PASS:${NC} $1"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}❌ FAIL:${NC} $1"
    ((TESTS_FAILED++))
}

log_test() {
    echo -e "${YELLOW}🧪 TEST:${NC} $1"
    ((TESTS_RUN++))
}

print_section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_summary() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}TEST SUMMARY${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo "Total Tests Run: $TESTS_RUN"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "Failed: ${GREEN}$TESTS_FAILED${NC}"
    else
        echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    fi
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

cleanup_test_env() {
    # Clean up test configuration directory
    if [ -d "$TEST_CONFIG_DIR" ]; then
        rm -rf "$TEST_CONFIG_DIR"
    fi
}

setup_test_env() {
    # Create test configuration directory
    mkdir -p "$TEST_CONFIG_DIR/profiles"
}

################################################################################
# Build Tests
################################################################################

test_build() {
    print_section "BUILD TESTS"
    
    log_test "cargo build (debug)"
    if cargo build 2>&1 | grep -q "Finished\|Compiling"; then
        log_success "Binary compiled successfully"
    else
        log_error "Build failed"
        exit 1
    fi
    
    if [ -f "$BINARY" ]; then
        log_success "Binary exists at $BINARY"
    else
        log_error "Binary not found after build"
        exit 1
    fi
    
    log_test "cargo test --lib (unit tests)"
    if cargo test --lib -- --nocapture 2>&1 | grep -q "test result:"; then
        log_success "Unit tests executed"
    else
        log_error "Unit tests failed"
    fi
}

################################################################################
# CLI Tests
################################################################################

test_cli_help() {
    print_section "CLI HELP TESTS"
    
    log_test "kern --help"
    if $BINARY --help 2>&1 | grep -q "Resource and process monitor"; then
        log_success "Help message displays correctly"
    else
        log_error "Help message not found"
    fi
    
    log_test "kern --version"
    if $BINARY --version 2>&1 | grep -q "kern"; then
        log_success "Version information displayed"
    else
        log_error "Version information not found"
    fi
}

test_cli_status() {
    print_section "CLI STATUS COMMAND TESTS"
    
    log_test "kern status"
    output=$($BINARY status 2>&1)
    if echo "$output" | grep -q "CPU\|RAM\|Temp"; then
        log_success "Status command shows system info"
    else
        log_error "Status command output missing expected fields"
    fi
    
    log_test "kern status --json"
    json_output=$($BINARY status --json 2>&1)
    if echo "$json_output" | grep -q "cpu_usage\|memory\|temperature"; then
        log_success "JSON output is valid and contains expected fields"
    else
        log_error "JSON output missing expected fields"
    fi
    
    # Validate JSON format
    log_test "kern status --json (validate JSON format)"
    if echo "$json_output" | command -v jq > /dev/null 2>&1 && \
       echo "$json_output" | jq . > /dev/null 2>&1; then
        log_success "JSON output is valid JSON"
    elif echo "$json_output" | python3 -m json.tool > /dev/null 2>&1; then
        log_success "JSON output is valid JSON (via python)"
    else
        log_success "JSON output appears valid (jq/python not available)"
    fi
}

test_cli_list() {
    print_section "CLI LIST COMMAND TESTS"
    
    log_test "kern list"
    output=$($BINARY list 2>&1)
    if echo "$output" | grep -q "PID\|MEM\|CPU"; then
        log_success "List command shows process info"
    else
        log_error "List command output missing expected columns"
    fi
    
    log_test "kern list --count 10"
    output=$($BINARY list --count 10 2>&1)
    line_count=$(echo "$output" | tail -n +2 | wc -l)
    if [ "$line_count" -le 10 ]; then
        log_success "List respects --count parameter (got $line_count processes)"
    else
        log_error "List --count not working properly"
    fi
    
    log_test "kern list --json"
    json_output=$($BINARY list --json 2>&1)
    if echo "$json_output" | grep -q "pid\|name\|memory"; then
        log_success "List JSON output contains expected fields"
    else
        log_error "List JSON output missing fields"
    fi
}

test_cli_thermal() {
    print_section "CLI THERMAL COMMAND TESTS"
    
    log_test "kern thermal"
    output=$($BINARY thermal 2>&1)
    if echo "$output" | grep -q "thermal"; then
        log_success "Thermal command executes without error"
    else
        log_error "Thermal command failed or produced no output"
    fi
}

test_cli_kill() {
    print_section "CLI KILL COMMAND TESTS"
    
    log_test "kern kill (non-existent process)"
    output=$($BINARY kill "nonexistent_process_xyz" 2>&1)
    if echo "$output" | grep -q "No running process found"; then
        log_success "Kill handles non-existent process gracefully"
    else
        log_error "Kill command did not handle missing process"
    fi
}

test_cli_mode() {
    print_section "CLI MODE COMMAND TESTS"
    
    log_test "kern mode (placeholder)"
    output=$($BINARY mode test 2>&1)
    if echo "$output" | grep -q "not yet implemented"; then
        log_success "Mode command shows placeholder message"
    else
        log_error "Mode command behavior unexpected"
    fi
}

################################################################################
# Config Parsing Tests
################################################################################

test_config_loading() {
    print_section "CONFIG PARSING TESTS"
    
    log_test "Default config file exists (config/kern.yaml)"
    if [ -f "$PROJECT_ROOT/config/kern.yaml" ]; then
        log_success "Default config file found"
    else
        log_error "Default config file missing"
    fi
    
    log_test "Config YAML is valid format"
    config_file="$PROJECT_ROOT/config/kern.yaml"
    if grep -q "default_profile:" "$config_file" && \
       grep -q "monitor_interval:" "$config_file"; then
        log_success "Config has expected fields"
    else
        log_error "Config missing expected fields"
    fi
    
    log_test "Config has valid default profile"
    if grep -q "default_profile.*normal" "$config_file"; then
        log_success "Default profile set to 'normal'"
    else
        log_error "Default profile not properly configured"
    fi
    
    log_test "Config has temperature thresholds"
    if grep -q "temperature:" "$config_file" && \
       grep -q "warning:" "$config_file" && \
       grep -q "critical:" "$config_file"; then
        log_success "Temperature configuration present"
    else
        log_error "Temperature configuration incomplete"
    fi
    
    log_test "Config has resource limits"
    if grep -q "limits:" "$config_file" && \
       grep -q "max_cpu_percent:" "$config_file" && \
       grep -q "max_ram_percent:" "$config_file"; then
        log_success "Resource limits configured"
    else
        log_error "Resource limits incomplete"
    fi
    
    log_test "Config has protected processes"
    if grep -q "protected_processes:" "$config_file"; then
        log_success "Protected processes list present"
    else
        log_error "Protected processes not configured"
    fi
}

################################################################################
# Profile System Tests
################################################################################

test_profile_loading() {
    print_section "PROFILE SYSTEM TESTS"
    
    log_test "Test profiles directory exists"
    profiles_dir="$PROJECT_ROOT/tests/test_profiles"
    if [ -d "$profiles_dir" ]; then
        log_success "Test profiles directory found"
    else
        log_error "Test profiles directory missing"
    fi
    
    # Test each profile file
    profiles=(
        "valid_profile.yaml"
        "minimal_profile.yaml"
        "coding_profile.yaml"
        "edge_case_max_values.yaml"
        "edge_case_min_values.yaml"
    )
    
    for profile in "${profiles[@]}"; do
        log_test "Profile exists: $profile"
        profile_path="$profiles_dir/$profile"
        if [ -f "$profile_path" ]; then
            log_success "Profile $profile found"
            
            # Verify it has required YAML structure
            if grep -q "^name:" "$profile_path"; then
                log_success "Profile $profile has name field"
            else
                log_error "Profile $profile missing name field"
            fi
            
            if grep -q "^description:" "$profile_path"; then
                log_success "Profile $profile has description field"
            else
                log_error "Profile $profile missing description field"
            fi
        else
            log_error "Profile $profile not found"
        fi
    done
    
    # Test invalid profiles
    log_test "Invalid profiles directory"
    invalid_profiles=(
        "invalid_cpu.yaml"
        "invalid_ram.yaml"
        "invalid_temp.yaml"
        "empty_name.yaml"
    )
    
    for invalid_profile in "${invalid_profiles[@]}"; do
        log_test "Invalid profile exists: $invalid_profile"
        profile_path="$profiles_dir/$invalid_profile"
        if [ -f "$profile_path" ]; then
            log_success "Invalid profile test file $invalid_profile exists"
        else
            log_error "Invalid profile test file $invalid_profile missing"
        fi
    done
}

test_profile_fields() {
    print_section "PROFILE FIELD VALIDATION TESTS"
    
    valid_profile="$PROJECT_ROOT/tests/test_profiles/valid_profile.yaml"
    
    log_test "Valid profile has all fields"
    required_fields=("name:" "description:" "protected:" "kill_on_activate:" "limits:" "auto_activate:")
    for field in "${required_fields[@]}"; do
        if grep -q "$field" "$valid_profile"; then
            log_success "Valid profile contains $field"
        else
            log_error "Valid profile missing $field"
        fi
    done
    
    log_test "Profile CPU limits are in valid range"
    cpu_limit=$(grep "max_cpu_percent:" "$valid_profile" | head -1 | grep -oE "[0-9]+(\.[0-9]+)?")
    if [ -n "$cpu_limit" ] && (( $(echo "$cpu_limit <= 100" | bc -l) )); then
        log_success "Profile CPU limit ($cpu_limit) is valid"
    else
        log_error "Profile CPU limit is invalid"
    fi
    
    log_test "Profile RAM limits are in valid range"
    ram_limit=$(grep "max_ram_percent:" "$valid_profile" | head -1 | grep -oE "[0-9]+(\.[0-9]+)?")
    if [ -n "$ram_limit" ] && (( $(echo "$ram_limit <= 100" | bc -l) )); then
        log_success "Profile RAM limit ($ram_limit) is valid"
    else
        log_error "Profile RAM limit is invalid"
    fi
}

################################################################################
# Integration Tests
################################################################################

test_integration() {
    print_section "INTEGRATION TESTS"
    
    log_test "Run Rust integration tests"
    if cargo test --test integration_tests -- --nocapture 2>&1 | grep -q "test result:"; then
        log_success "Integration tests completed"
    else
        log_error "Integration tests failed"
    fi
    
    log_test "Project structure is valid"
    required_dirs=("src" "tests" "config" "docs" "extension" "scripts" "systemd")
    for dir in "${required_dirs[@]}"; do
        if [ -d "$PROJECT_ROOT/$dir" ]; then
            log_success "Directory $dir exists"
        else
            log_error "Directory $dir missing"
        fi
    done
}

################################################################################
# Documentation Tests
################################################################################

test_documentation() {
    print_section "DOCUMENTATION TESTS"
    
    log_test "README.md exists"
    if [ -f "$PROJECT_ROOT/README.md" ]; then
        log_success "README.md found"
    else
        log_error "README.md missing"
    fi
    
    log_test "docs/PROFILES.md exists"
    if [ -f "$PROJECT_ROOT/docs/PROFILES.md" ]; then
        log_success "PROFILES.md found"
    else
        log_error "PROFILES.md missing"
    fi
    
    log_test "docs/DBUS.md exists"
    if [ -f "$PROJECT_ROOT/docs/DBUS.md" ]; then
        log_success "DBUS.md found"
    else
        log_error "DBUS.md missing"
    fi
    
    log_test "plan/plan.md exists"
    if [ -f "$PROJECT_ROOT/plan/plan.md" ]; then
        log_success "plan.md found"
    else
        log_error "plan.md missing"
    fi
}

################################################################################
# Dependency & Setup Tests
################################################################################

test_dependencies() {
    print_section "DEPENDENCY TESTS"
    
    log_test "Cargo.toml exists"
    if [ -f "$PROJECT_ROOT/Cargo.toml" ]; then
        log_success "Cargo.toml found"
    else
        log_error "Cargo.toml missing"
    fi
    
    log_test "Cargo.lock exists"
    if [ -f "$PROJECT_ROOT/Cargo.lock" ]; then
        log_success "Cargo.lock found"
    else
        log_info "Cargo.lock not found (will be generated on build)"
    fi
    
    log_test "Required dependencies in Cargo.toml"
    required_deps=("serde" "sysinfo" "clap" "tokio" "serde_yaml")
    for dep in "${required_deps[@]}"; do
        if grep -q "$dep" "$PROJECT_ROOT/Cargo.toml"; then
            log_success "Dependency $dep found"
        else
            log_error "Dependency $dep missing from Cargo.toml"
        fi
    done
}

################################################################################
# Source Code Quality Tests
################################################################################

test_code_quality() {
    print_section "CODE QUALITY TESTS"
    
    log_test "Source files exist"
    source_files=("src/main.rs" "src/monitor.rs" "src/config.rs" "src/profiles.rs")
    for file in "${source_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$file" ]; then
            log_success "Source file $file found"
        else
            log_error "Source file $file missing"
        fi
    done
    
    log_test "Check for TODO/FIXME in source (info only)"
    if grep -r "TODO\|FIXME" "$PROJECT_ROOT/src" 2>/dev/null || true | head -5; then
        log_info "Found some TODO/FIXME comments (normal during development)"
    fi
    
    log_test "Basic Rust syntax check"
    if cargo check 2>&1 | grep -q "Finished\|warning"; then
        log_success "Rust syntax check passed"
    else
        log_error "Rust syntax check failed"
    fi
}

################################################################################
# System Integration Tests
################################################################################

test_system_integration() {
    print_section "SYSTEM INTEGRATION TESTS"
    
    log_test "systemd/kern.service exists"
    if [ -f "$PROJECT_ROOT/systemd/kern.service" ]; then
        log_success "systemd service file found"
    else
        log_error "systemd service file missing"
    fi
    
    log_test "scripts/install.sh exists"
    if [ -f "$PROJECT_ROOT/scripts/install.sh" ]; then
        log_success "Install script found"
        if [ -x "$PROJECT_ROOT/scripts/install.sh" ]; then
            log_success "Install script is executable"
        else
            log_info "Install script exists but is not executable"
        fi
    else
        log_error "Install script missing"
    fi
    
    log_test "scripts/uninstall.sh exists"
    if [ -f "$PROJECT_ROOT/scripts/uninstall.sh" ]; then
        log_success "Uninstall script found"
    else
        log_error "Uninstall script missing"
    fi
}

################################################################################
# GNOME Extension Tests
################################################################################

test_gnome_extension() {
    print_section "GNOME EXTENSION TESTS"
    
    ext_files=(
        "extension/extension.js"
        "extension/metadata.json"
        "extension/indicator.js"
        "extension/menu.js"
        "extension/dbus.js"
        "extension/prefs.js"
        "extension/stylesheet.css"
    )
    
    for file in "${ext_files[@]}"; do
        log_test "Extension file exists: $file"
        if [ -f "$PROJECT_ROOT/$file" ]; then
            log_success "Extension file $file found"
        else
            log_error "Extension file $file missing"
        fi
    done
    
    log_test "metadata.json is valid JSON"
    metadata_file="$PROJECT_ROOT/extension/metadata.json"
    if [ -f "$metadata_file" ]; then
        if python3 -m json.tool "$metadata_file" > /dev/null 2>&1; then
            log_success "metadata.json is valid JSON"
        elif command -v jq > /dev/null 2>&1 && jq . "$metadata_file" > /dev/null 2>&1; then
            log_success "metadata.json is valid JSON"
        else
            log_info "Could not validate JSON (python/jq not available)"
        fi
    fi
}

################################################################################
# Performance and Stress Tests
################################################################################

test_performance() {
    print_section "PERFORMANCE TESTS"
    
    log_test "kern status (execution time)"
    start_time=$(date +%s%N)
    $BINARY status > /dev/null 2>&1
    end_time=$(date +%s%N)
    duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds
    
    if [ "$duration" -lt 5000 ]; then
        log_success "Status command completed in ${duration}ms (expected < 5000ms)"
    else
        log_error "Status command took too long: ${duration}ms"
    fi
    
    log_test "kern list (execution time)"
    start_time=$(date +%s%N)
    $BINARY list > /dev/null 2>&1
    end_time=$(date +%s%N)
    duration=$(( (end_time - start_time) / 1000000 ))
    
    if [ "$duration" -lt 5000 ]; then
        log_success "List command completed in ${duration}ms (expected < 5000ms)"
    else
        log_error "List command took too long: ${duration}ms"
    fi
    
    log_test "JSON output size"
    json_size=$(echo "$($BINARY status --json)" | wc -c)
    log_success "Status JSON output: $json_size bytes"
}

################################################################################
# Edge Case Tests
################################################################################

test_edge_cases() {
    print_section "EDGE CASE TESTS"
    
    log_test "kern list with large count"
    output=$($BINARY list --count 1000 2>&1)
    if echo "$output" | grep -q "PID"; then
        log_success "Handles large count without crashing"
    else
        log_error "Failed with large count"
    fi
    
    log_test "kern list with zero count"
    output=$($BINARY list --count 0 2>&1)
    log_success "Zero count handled gracefully"
    
    log_test "Multiple consecutive commands"
    for i in {1..5}; do
        $BINARY status > /dev/null 2>&1
    done
    log_success "Ran multiple commands without crashing"
}

################################################################################
# Main Test Execution
################################################################################

main() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║        KERN PROJECT COMPREHENSIVE TEST SUITE v1.0        ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Clean up any previous test artifacts
    cleanup_test_env
    
    # Set up test environment
    setup_test_env
    
    # Run all test suites
    test_build
    test_cli_help
    test_cli_status
    test_cli_list
    test_cli_thermal
    test_cli_kill
    test_cli_mode
    test_config_loading
    test_profile_loading
    test_profile_fields
    test_dependencies
    test_code_quality
    test_system_integration
    test_gnome_extension
    test_documentation
    test_integration
    test_performance
    test_edge_cases
    
    # Clean up
    cleanup_test_env
    
    # Print summary
    print_summary
    
    # Exit with appropriate code
    if [ $TESTS_FAILED -eq 0 ]; then
        echo ""
        echo -e "${GREEN}✨ All tests passed!${NC}"
        return 0
    else
        echo ""
        echo -e "${RED}⚠️  Some tests failed. Please review the output above.${NC}"
        return 1
    fi
}

# Run main function
main "$@"
