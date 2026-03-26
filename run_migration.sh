#!/bin/bash
set -e

export DB_SCHEMA="canary"

if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL environment variable is not set"
    exit 1
fi

# Modify DATABASE_URL to set search_path
if [[ $DATABASE_URL == *"?"* ]]; then
    # URL already has query parameters
    export DATABASE_URL="${DATABASE_URL}&options=-c%20search_path%3D${DB_SCHEMA}%2Cpublic"
else
    # URL has no query parameters
    export DATABASE_URL="${DATABASE_URL}?options=-c%20search_path%3D${DB_SCHEMA}%2Cpublic"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Running Canary Migrations"
echo "Schema: $DB_SCHEMA"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run custom migration binary
cargo run --manifest-path migration/Cargo.toml --bin run_canary_migrations

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Migration complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
