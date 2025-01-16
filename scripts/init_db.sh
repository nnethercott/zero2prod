set -x # lists all commands run
set -eo pipefail # something about nonzero exit

# check we have dependencies installed
if ![-x "$(command -v psql)"]; then
    echo >&2 "Error; psql not installed!"
    exit 1
fi

if ![-x "$(command -v sqlx)"]; then
    echo >&2 "Error; sqlx not installed!"
    echo >&2 "Use: `cargo install sqlx-cli --no-default-features --features native-tls,postgres`"
    exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]]  # -z checks if string is null
then
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -e POSTGRES_HOST=${DB_HOST} \
        -e POSTGRES_PORT=${DB_PORT} \
        -p ${DB_PORT}:5432 \
        -d postgres \
        postgres -N 1000
fi

# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "postgres is up and running !"
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAM}

sqlx database create --database-url ${DATABASE_URL}
