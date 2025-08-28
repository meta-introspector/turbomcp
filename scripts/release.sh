#!/bin/bash

# TurboMCP Comprehensive Release Script
# 
# This script handles the complete release process for TurboMCP:
# - Pre-flight checks and validation
# - Code quality checks
# - Version consistency verification
# - Crate packaging and publishing
# - Post-release tasks
#
# Environment Variables:
#   DRY_RUN               - Set to 'false' to perform actual publishing (default: true)
#   VERSION               - Override version to publish (auto-detected from Cargo.toml)
#   DELAY_BETWEEN_PUBLISHES - Delay between crate publishes in seconds (default: 60)
#   VERIFICATION_RETRIES  - Number of verification attempts (default: 3)
#   SKIP_TESTS           - Set to 'true' to skip test execution (default: false)
#
# Usage:
#   ./scripts/release.sh                    # Dry run with all checks
#   DRY_RUN=false ./scripts/release.sh      # Full release
#   SKIP_TESTS=true ./scripts/release.sh    # Quick validation without tests

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

DRY_RUN=${DRY_RUN:-true}
SKIP_TESTS=${SKIP_TESTS:-false}
DELAY_BETWEEN_PUBLISHES=${DELAY_BETWEEN_PUBLISHES:-60}
VERIFICATION_RETRIES=${VERIFICATION_RETRIES:-3}

# Auto-detect version from main crate if not specified
if [ -z "${VERSION:-}" ]; then
    VERSION=$(grep '^version = ' "crates/turbomcp/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
    if [ -z "$VERSION" ]; then
        echo "❌ Could not auto-detect version from crates/turbomcp/Cargo.toml"
        exit 1
    fi
fi

# Crate publish order - CAREFULLY DETERMINED based on dependencies
# This order ensures dependencies are published before dependents
CRATES=(
    "turbomcp-core"      # Foundation - no internal deps
    "turbomcp-protocol"  # Depends on: turbomcp-core
    "turbomcp-macros"    # Depends on: turbomcp-core, turbomcp-protocol (dev-deps)
    "turbomcp-dpop"      # Depends on: turbomcp-core, turbomcp-protocol
    "turbomcp-transport" # Depends on: turbomcp-core, turbomcp-dpop (optional)
    "turbomcp-server"    # Depends on: turbomcp-core, turbomcp-protocol, turbomcp-transport, turbomcp-macros
    "turbomcp-client"    # Depends on: turbomcp-core, turbomcp-protocol, turbomcp-transport
    "turbomcp-cli"       # Standalone CLI tool - no internal deps
    "turbomcp"           # Main crate - depends on all others (optional)
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ============================================================================
# Utility Functions
# ============================================================================

print_header() {
    echo ""
    echo -e "${BOLD}${BLUE}🚀 $1${NC}"
    echo -e "${BLUE}==================================================${NC}"
}

print_section() {
    echo ""
    echo -e "${CYAN}📋 $1${NC}"
    echo -e "${CYAN}----------------------------------------${NC}"
}

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Detect best available search tool
detect_search_tool() {
    if command -v rg >/dev/null 2>&1; then
        echo "rg"
    elif command -v grep >/dev/null 2>&1; then
        echo "grep"
    else
        print_error "No search tool available (ripgrep or grep required)"
        exit 1
    fi
}

# Universal search function
search_pattern() {
    local pattern="$1"
    local path="$2"
    local tool=$(detect_search_tool)
    
    case "$tool" in
        "rg")
            rg "$pattern" --type rust "$path" --count-matches 2>/dev/null | awk -F: '{sum+=$2} END {print (sum ? sum : 0)}'
            ;;
        "grep")
            find "$path" -name "*.rs" -exec grep -l "$pattern" {} \; 2>/dev/null | wc -l | tr -d ' '
            ;;
    esac
}

# Check if crate exists on crates.io
check_crate_published() {
    local crate_name="$1"
    local version="$2"
    
    # Try to get crate info from crates.io API
    if command -v curl >/dev/null 2>&1; then
        local response=$(curl -s "https://crates.io/api/v1/crates/$crate_name" || echo "")
        if echo "$response" | grep -q "\"max_version\":\"$version\""; then
            return 0  # Found
        fi
    fi
    
    # Fallback to cargo search
    if cargo search --limit 1 "$crate_name" 2>/dev/null | grep -q "$crate_name.*$version"; then
        return 0  # Found
    fi
    
    return 1  # Not found
}

# ============================================================================
# Main Script
# ============================================================================

print_header "TurboMCP Release Script v2.0"

echo -e "${BOLD}Configuration:${NC}"
echo "  Version: ${BOLD}$VERSION${NC}"
echo "  Dry Run: ${BOLD}$DRY_RUN${NC}"
echo "  Skip Tests: ${BOLD}$SKIP_TESTS${NC}"
echo "  Delay Between Publishes: ${BOLD}${DELAY_BETWEEN_PUBLISHES}s${NC}"
echo "  Verification Retries: ${BOLD}$VERIFICATION_RETRIES${NC}"

if [ "$DRY_RUN" = "true" ]; then
    echo ""
    print_warning "DRY RUN MODE - No actual publishing will occur"
    print_info "Set DRY_RUN=false to perform actual release"
fi

# ============================================================================
# Pre-flight Checks
# ============================================================================

print_section "Pre-flight Environment Checks"

# Check workspace location
if [ ! -f "Cargo.toml" ] || [ ! -d "crates" ]; then
    print_error "Must be run from the turbomcp workspace root directory"
    exit 1
fi

# Check required tools
required_tools=("cargo")
missing_tools=()

for tool in "${required_tools[@]}"; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        missing_tools+=("$tool")
    fi
done

if [ ${#missing_tools[@]} -ne 0 ]; then
    print_error "Missing required tools: ${missing_tools[*]}"
    exit 1
fi

# Check cargo credentials for actual publishing
if [ "$DRY_RUN" = "false" ]; then
    if [ ! -f ~/.cargo/credentials.toml ] && [ ! -f ~/.cargo/credentials ]; then
        print_error "Not logged into crates.io. Run 'cargo login' first"
        exit 1
    fi
fi

print_status "Environment checks passed"

# ============================================================================
# Workspace Validation
# ============================================================================

print_section "Workspace Validation"

# Verify all expected crates exist
missing_crates=()
for crate in "${CRATES[@]}"; do
    if [ ! -d "crates/$crate" ] || [ ! -f "crates/$crate/Cargo.toml" ]; then
        missing_crates+=("$crate")
    fi
done

if [ ${#missing_crates[@]} -ne 0 ]; then
    print_error "Missing crates: ${missing_crates[*]}"
    exit 1
fi

print_status "All expected crates found"

# Clean workspace for fresh build
print_info "Cleaning workspace..."
cargo clean >/dev/null 2>&1

# ============================================================================
# Version Consistency Check
# ============================================================================

print_section "Version Consistency Check"

version_issues=0
for crate in "${CRATES[@]}"; do
    crate_version=$(grep '^version = ' "crates/$crate/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
    if [ "$crate_version" != "$VERSION" ]; then
        print_error "$crate has version '$crate_version', expected '$VERSION'"
        version_issues=$((version_issues + 1))
    fi
    
    # Check internal dependency versions
    while IFS= read -r line; do
        if echo "$line" | grep -q "turbomcp.*version.*="; then
            dep_version=$(echo "$line" | sed 's/.*version = "\([^"]*\)".*/\1/')
            if [ "$dep_version" != "$VERSION" ]; then
                print_error "$crate has dependency version '$dep_version', expected '$VERSION'"
                version_issues=$((version_issues + 1))
            fi
        fi
    done < "crates/$crate/Cargo.toml"
done

if [ $version_issues -eq 0 ]; then
    print_status "All crates have consistent version $VERSION"
else
    print_error "$version_issues version inconsistencies found"
    exit 1
fi

# ============================================================================
# Compilation Check
# ============================================================================

print_section "Compilation Verification"

print_info "Checking workspace compilation..."
if cargo check --workspace --all-targets --quiet; then
    print_status "All crates compile successfully"
else
    print_error "Compilation failed"
    exit 1
fi

# ============================================================================
# Code Quality Checks
# ============================================================================

print_section "Code Quality Validation"

# Format check
print_info "Checking code formatting..."
if cargo fmt --all -- --check >/dev/null 2>&1; then
    print_status "Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt --all' to fix"
    exit 1
fi

# Clippy check
print_info "Running clippy lints..."
if cargo clippy --workspace --all-targets --quiet -- -D clippy::correctness -D clippy::suspicious -D clippy::complexity -A clippy::too_many_arguments -A clippy::type_complexity; then
    print_status "Clippy checks passed"
else
    print_error "Clippy issues found"
    exit 1
fi

# Code quality patterns check
print_info "Scanning for problematic code patterns..."
search_tool=$(detect_search_tool)
print_info "Using search tool: $search_tool"

todo_count=$(search_pattern "TODO.*implement|TODO.*stub|not_implemented|unimplemented!\(\)" "crates/")
if [ "$todo_count" -gt 10 ]; then
    print_warning "Found $todo_count critical TODOs/stubs (acceptable for experimental release)"
    print_info "To inspect: find crates/ -name '*.rs' -exec grep -n 'TODO.*implement\\|TODO.*stub\\|not_implemented\\|unimplemented!' {} +"
else
    print_status "Code quality check passed ($todo_count TODOs found)"
fi

# ============================================================================
# Test Execution
# ============================================================================

if [ "$SKIP_TESTS" = "false" ]; then
    print_section "Test Suite Execution"
    
    print_info "Running workspace tests..."
    if cargo test --workspace --lib --quiet; then
        print_status "All tests pass"
    else
        print_error "Test failures detected"
        exit 1
    fi
else
    print_section "Test Suite (SKIPPED)"
    print_warning "Tests skipped as requested"
fi

# ============================================================================
# Documentation Generation
# ============================================================================

print_section "Documentation Generation"

print_info "Generating documentation..."
if cargo doc --workspace --no-deps --quiet; then
    print_status "Documentation generated successfully"
else
    print_warning "Documentation generation had issues (non-fatal)"
fi

# ============================================================================
# Crate Metadata Validation
# ============================================================================

print_section "Crate Metadata Validation"

metadata_issues=0
required_fields=("description" "license" "repository")

for crate in "${CRATES[@]}"; do
    cargo_toml="crates/$crate/Cargo.toml"
    missing_fields=()
    
    for field in "${required_fields[@]}"; do
        if ! grep -q "^$field = " "$cargo_toml"; then
            missing_fields+=("$field")
        fi
    done
    
    if [ ${#missing_fields[@]} -ne 0 ]; then
        print_error "$crate missing required metadata: ${missing_fields[*]}"
        metadata_issues=$((metadata_issues + 1))
    fi
    
    # Check for README
    if [ ! -f "crates/$crate/README.md" ]; then
        print_warning "$crate missing README.md (recommended but not required)"
    fi
done

if [ $metadata_issues -eq 0 ]; then
    print_status "All crates have required metadata"
else
    print_error "$metadata_issues crates have metadata issues"
    exit 1
fi

# ============================================================================
# Package Verification
# ============================================================================

print_section "Package Verification"

for crate in "${CRATES[@]}"; do
    print_info "Packaging $crate..."
    # Use --no-verify for experimental versions since dependencies may not be on crates.io yet
    if cargo package --manifest-path "crates/$crate/Cargo.toml" --quiet --no-verify --allow-dirty; then
        print_status "$crate packaged successfully"
    else
        print_error "Failed to package $crate"
        # For experimental versions, show the error but don't exit immediately
        if [[ "$VERSION" == *"exp"* ]] || [[ "$VERSION" == *"alpha"* ]] || [[ "$VERSION" == *"beta"* ]] || [[ "$VERSION" == *"rc"* ]]; then
            print_warning "Packaging failed for $crate - this might be expected for experimental versions"
            print_info "Continuing with other crates..."
        else
            exit 1
        fi
    fi
done

# ============================================================================
# Git Status Check
# ============================================================================

print_section "Git Status Verification"

if command -v git >/dev/null 2>&1; then
    if [ -n "$(git status --porcelain 2>/dev/null || true)" ]; then
        print_warning "Uncommitted changes detected:"
        git status --short 2>/dev/null || true
        echo ""
        print_info "Consider committing changes before release"
    else
        print_status "Working directory is clean"
    fi
else
    print_info "Git not available, skipping status check"
fi

# ============================================================================
# Publishing Phase
# ============================================================================

print_section "Publishing Phase"

if [ "$DRY_RUN" = "true" ]; then
    echo ""
    print_info "DRY RUN: Publishing simulation"
    echo ""
    echo "Would publish crates in the following order:"
    for i in "${!CRATES[@]}"; do
        echo "$((i+1)). ${CRATES[$i]} (v$VERSION)"
    done
    echo ""
    print_info "To perform actual publishing:"
    echo "  DRY_RUN=false $0"
else
    echo ""
    print_info "Publishing crates to crates.io..."
    echo ""
    echo "Publishing order:"
    for i in "${!CRATES[@]}"; do
        echo "$((i+1)). ${CRATES[$i]}"
    done
    echo ""
    
    # Confirmation prompt
    read -p "Continue with publishing? [y/N]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Publishing aborted by user"
        exit 0
    fi
    
    # Publish each crate
    published_crates=()
    failed_crates=()
    
    for i in "${!CRATES[@]}"; do
        crate="${CRATES[$i]}"
        echo ""
        print_info "Publishing $((i+1))/${#CRATES[@]}: $crate"
        
        # Check if already published at this version
        if check_crate_published "$crate" "$VERSION"; then
            print_warning "$crate v$VERSION already published, skipping"
            published_crates+=("$crate")
            continue
        fi
        
        # Attempt to publish
        if timeout 300 cargo publish --manifest-path "crates/$crate/Cargo.toml"; then
            print_status "$crate published successfully"
            published_crates+=("$crate")
            
            # Verify publication
            print_info "Verifying publication on crates.io..."
            for attempt in $(seq 1 $VERIFICATION_RETRIES); do
                sleep 5  # Brief pause before verification
                if check_crate_published "$crate" "$VERSION"; then
                    print_status "$crate verified on crates.io"
                    break
                else
                    if [ $attempt -lt $VERIFICATION_RETRIES ]; then
                        print_info "Verification attempt $attempt failed, retrying..."
                    else
                        print_warning "$crate published but verification failed (usually fine)"
                    fi
                fi
            done
        else
            print_error "Failed to publish $crate"
            failed_crates+=("$crate")
            
            # Ask whether to continue with remaining crates
            echo ""
            print_warning "Publication of $crate failed. This might be due to:"
            echo "  - Network issues or timeouts"
            echo "  - Rate limiting by crates.io"
            echo "  - Dependency resolution problems"
            echo "  - Crate already published"
            echo ""
            read -p "Continue with remaining crates? [y/N]: " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                print_error "Publishing stopped due to failure"
                exit 1
            fi
        fi
        
        # Wait between publishes (except for the last one)
        if [ $i -lt $((${#CRATES[@]} - 1)) ]; then
            print_info "Waiting $DELAY_BETWEEN_PUBLISHES seconds for crates.io processing..."
            sleep $DELAY_BETWEEN_PUBLISHES
        fi
    done
    
    # Publishing summary
    echo ""
    print_info "Publishing Summary:"
    echo "  Successfully published: ${#published_crates[@]}/${#CRATES[@]} crates"
    if [ ${#failed_crates[@]} -gt 0 ]; then
        echo "  Failed crates: ${failed_crates[*]}"
    fi
fi

# ============================================================================
# Release Summary & Next Steps
# ============================================================================

print_header "Release Summary"

echo -e "${BOLD}Release Details:${NC}"
echo "  Version: $VERSION"
echo "  Total Crates: ${#CRATES[@]}"
echo "  Publish Order: ${CRATES[*]}"

if [ "$DRY_RUN" = "true" ]; then
    echo ""
    print_info "This was a dry run. No actual publishing occurred."
    echo ""
    echo -e "${BOLD}Next steps:${NC}"
    echo "1. Review any warnings or issues above"
    echo "2. Commit any final changes to git"
    echo "3. Create and push a git tag:"
    echo "   git tag v$VERSION"
    echo "   git push origin v$VERSION"
    echo "4. Run actual publish:"
    echo "   DRY_RUN=false $0"
    echo "5. Create GitHub release with changelog"
else
    if [ ${#failed_crates[@]} -eq 0 ]; then
        echo ""
        print_status "🎉 All crates published successfully!"
    else
        echo ""
        print_warning "Release completed with some failures"
    fi
    
    echo ""
    echo -e "${BOLD}Post-release tasks:${NC}"
    echo "1. Create and push git tag (if not done):"
    echo "   git tag v$VERSION"
    echo "   git push origin v$VERSION"
    echo "2. Create GitHub release with changelog"
    echo "3. Update documentation and announcements"
    echo "4. Monitor crates.io for successful indexing"
    echo "5. Test installation: cargo install turbomcp-cli --version $VERSION"
fi

echo ""
print_info "Release script completed"