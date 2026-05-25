#!/bin/sh
set -e

psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -c "CREATE DATABASE retailer_sourcing;"
psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -c "CREATE DATABASE retailer_sourcing_test;"
