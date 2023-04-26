#!/usr/bin/env bash

set -x
set -eo pipefail

if ! [ -x "$(command -v surreal)" ]; then
  echo 'Error: surreal is not installed.' >&2
  exit 1
fi

if ! [ -x "$(command -v surrealdb-migrations)" ]; then
  echo 'Error: surrealdb-migrations is not installed.' >&2
  exit 1
fi

DB_USER="${SURREAL_USER:=surreal}"
DB_PASSWORD="${SURREAL_PASSWORD:=password}"
DB_NAME="${SURREAL_DB:=newsletter}"
DB_PORT="${SURREAL_PORT:=8000}"
DB_HOST="${SURREAL_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]]
then
  docker run \
    --rm \
    --pull always \
    -p "${DB_PORT}":8000 \
    -d \
    surrealdb/surrealdb:latest start \
      --log trace \
      --user "${DB_USER}" \
      --pass "${DB_PASSWORD}" \
      memory
fi

export DBPASSWORD="${DB_PASSWORD}"
until surreal isready --conn http://${DB_HOST}:${DB_PORT}; do
  >&2 echo "SurrealDB is unavailable - sleeping"
  sleep 1
done

>&2 echo "SurrealDB is up and running on port ${DB_PORT}!"
DATABASE_URL=http://${DB_HOST}:${DB_PORT}
export DATABASE_URL

surrealdb-migrations apply

>&2 echo "SurrealDB migrations applied! Let's Go!!!!"