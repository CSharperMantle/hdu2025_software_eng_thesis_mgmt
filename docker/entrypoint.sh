#!/bin/sh

set -e

# Parse DATABASE_URL for pg_isready and psql
# Format: postgres://user:password@host:port/dbname
tmp="${DATABASE_URL#*://}"
auth="${tmp%@*}"
location="${tmp##*@}"

db_user="${auth%%:*}"
db_password="${auth#*:}"

host_port="${location%%/*}"
db_name="${location#*/}"
db_name="${db_name%%\?*}"

db_host="${host_port%%:*}"
if echo "$host_port" | grep -q ":"; then
	db_port="${host_port#*:}"
else
	db_port="5432"
fi

echo 'Waiting for PostgreSQL to be ready...'
until pg_isready -h "$db_host" -p "$db_port" -U "$db_user"; do
	sleep 1
done

echo 'Running database migrations...'
diesel migration run

if [ -f '/docker-entrypoint-initdb.d/init.sql' ]; then
	echo 'Running init SQL script...'
	PGPASSWORD="$db_password" psql -h "$db_host" -U "$db_user" -d "$db_name" -f /docker-entrypoint-initdb.d/init.sql || true
fi

echo 'Starting server...'
exec backend_server
