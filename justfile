# Load the .env automatically
set dotenv-load

# --unsorted so that we list them in the order they're in the file
default:
  @just --list --unsorted

# build all
build-all:
	cargo build --all-targets

# release build all
build-all-release:
	cargo build --all-targets --release

# build
build:
	cargo build

# run the tests
test:
	RUST_BACKTRACE=1 cargo test

# run the unit tests w/ a release binary
test-release:
	RUST_BACKTRACE=1 cargo test --release

# Run the backend server
run-server-release:
	export RUST_LOG="petro=debug,info"
	cargo run --release

# Reset the database and migrate it
run-reset: compose-clean compose sleep migrate-database

sleep:
	sleep 5

LOCAL_USER_ID := `id -u`

# Run development environment
compose:
	LOCAL_USER_ID={{LOCAL_USER_ID}} docker compose up -d

# Run development environment
compose-inspect:
	LOCAL_USER_ID={{LOCAL_USER_ID}} docker compose up

# Remove dev env containers
compose-rm OPTS:
	LOCAL_USER_ID={{LOCAL_USER_ID}} docker compose rm {{OPTS}}

# Nukes dev env entirely (NOTICE: removed data volume as well)
compose-clean:
	LOCAL_USER_ID={{LOCAL_USER_ID}} docker compose down -v

# bring down the database - keep volume
compose-down:
	LOCAL_USER_ID={{LOCAL_USER_ID}} docker compose down

# The quotes delay the evaluation of the command until it's run
# backticks only would cause the command to be evaluated immediately
cols := "`tput cols`"
lines := "`tput lines`"
database_container := "`docker compose ps -q pg`"
etl_database_container := "`docker compose ps -q pg_etl`"
database_username := "${DATABASE_USERNAME}"
database_name := "${DATABASE_NAME}"

# Connect to the backend dev database via psql
psql:
	@docker exec -e COLUMNS="{{cols}}" -e LINES="{{lines}}" -it {{database_container}} /bin/bash -c "reset -w && psql -U{{database_username}} {{database_name}}"

# Show the command to connect to the backend dev database via psql
show-psql:
	echo 'docker exec -e COLUMNS="{{cols}}" -e LINES="{{lines}}" -it {{database_container}} /bin/bash -c "reset -w && psql -U{{database_username}} {{database_name}}"'

# Migrate dev data using diesel
migrate-database:
	diesel migration run --database-url=${DATABASE_URL} --config-file=diesel.toml --migration-dir ./migrations

# generate a new migration for dev data, argument is the migration name like `create_users`
generate-migration OPTS:
	diesel migration generate --database-url=${DATABASE_URL} --config-file=diesel.toml --migration-dir ./migrations {{OPTS}}

# Insert example user via the web API
insert-example-user:
	curl -X POST http://localhost:8080/user -H "Content-Type: application/json" -d '{"name": "Chris"}'

# Install diesel CLI with the appropriate features
install-diesel:
	cargo install diesel_cli --version 2.1.1 --no-default-features --features postgres

# Install development dependencies
install-dev-dependencies: install-diesel

# cargo fmt
fmt:
	cargo +nightly fmt
