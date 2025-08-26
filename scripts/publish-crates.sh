#!/bin/bash

# TurboMCP Manual Publishing Script
# Quick script to publish all crates in the correct order

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Crate publish order (dependencies first)
CRATES=(
    "turbomcp-core"
    "turbomcp-macros"
    "turbomcp-protocol"
    "turbomcp-transport"
    "turbomcp-cli"
    "turbomcp-client"
    "turbomcp-server"
    "turbomcp"
)

echo -e "${BLUE}🚀 Publishing TurboMCP Crates${NC}"
echo "=============================="
echo ""

# Check if we're logged in by testing credentials file
if ! test -f ~/.cargo/credentials.toml; then
    echo -e "${YELLOW}⚠️  Not logged into crates.io. Please run 'cargo login' first${NC}"
    exit 1
fi

echo "Publishing order:"
for i in "${!CRATES[@]}"; do
    echo "$((i+1)). ${CRATES[$i]}"
done
echo ""

read -p "Continue with publishing? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

for i in "${!CRATES[@]}"; do
    crate="${CRATES[$i]}"
    echo ""
    echo -e "${BLUE}Publishing $((i+1))/${#CRATES[@]}: $crate${NC}"
    
    if cargo publish --manifest-path "crates/$crate/Cargo.toml"; then
        echo -e "${GREEN}✅ $crate published successfully${NC}"
    else
        echo -e "${RED}❌ Failed to publish $crate${NC}"
        exit 1
    fi
    
    # Wait between publishes (except for the last one)
    if [ $i -lt $((${#CRATES[@]} - 1)) ]; then
        echo "Waiting 30 seconds for crates.io to process..."
        sleep 30
    fi
done

echo ""
echo -e "${GREEN}🎉 All crates published successfully!${NC}"
echo ""
echo "Next steps:"
echo "1. Create git tag: git tag v1.0.0"
echo "2. Push tag: git push origin v1.0.0"
echo "3. Create GitHub release"
echo "4. Launch social media campaign!"