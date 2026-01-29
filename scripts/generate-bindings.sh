#!/bin/bash

# Generate TypeScript bindings for all Soroban contracts
# The bindings directory will be an npm workspace package containing all contract bindings

set -e

echo "Generating TypeScript bindings for Stellars Finance contracts..."

# Navigate to bindings directory
cd bindings

# Contract names
CONTRACTS=(
  "position-manager"
  "liquidity-pool"
  "config-manager"
  "market-manager"
  "oracle-integrator"
  "custom-oracle"
  "faucet-token"
)

# Clean existing contract bindings
echo "Cleaning old bindings..."
for contract in "${CONTRACTS[@]}"; do
  rm -rf "$contract"
done

# Generate bindings for each contract
for contract in "${CONTRACTS[@]}"; do
  echo "Generating bindings for $contract..."

  WASM_PATH="../contracts/target/wasm32v1-none/release/${contract//-/_}.wasm"

  if [ ! -f "$WASM_PATH" ]; then
    echo "Error: WASM file not found at $WASM_PATH"
    echo "Please run 'npm run build:contracts' first"
    exit 1
  fi

  stellar contract bindings typescript \
    --wasm "$WASM_PATH" \
    --output-dir "$contract"

  # Update package.json to use @stellars-finance scope
  if [ -f "$contract/package.json" ]; then
    sed -i '' "s/\"name\": \"$contract\"/\"name\": \"@stellars-finance\/$contract\"/" "$contract/package.json"
    echo "  → Updated package name to @stellars-finance/$contract"
  fi

  echo "✓ $contract bindings generated"
done

echo ""
echo "Installing dependencies for all bindings via workspace..."
cd ..
npm install

echo ""
echo "Building all binding packages..."
cd bindings
for contract in "${CONTRACTS[@]}"; do
  echo "Building $contract..."
  cd "$contract"
  npm run build
  cd ..
done

echo ""
echo "✓ All bindings generated successfully!"
echo ""
echo "Generated packages in bindings/:"
for contract in "${CONTRACTS[@]}"; do
  echo "  - $contract/"
done
