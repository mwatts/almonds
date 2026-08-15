[working-directory: 'kernel']
watch-kernel:
	cargo watch -qcx check

# Start postgres and mysql containers (kernel/docker/docker-compose.yml)
[working-directory: 'kernel']
db-up:
	docker compose -f docker/docker-compose.yml up -d

# Stop and remove containers (data volumes are preserved)
[working-directory: 'kernel']
db-down:
	docker compose -f docker/docker-compose.yml down

# Tear down containers AND delete volumes
[working-directory: 'kernel']
db-clean:
	docker compose -f docker/docker-compose.yml down -v

# --- Migrations -------------------------------------------------------



[working-directory: 'kernel']
@migrate-add target:
	@sea-orm-cli migrate generate "{{target}}"

[working-directory: 'kernel']
@migrate-init:
	@sea-orm-cli migrate init

# Run pending migrations against postgres (docker)
[working-directory: 'kernel']
@migrate-up-postgres:
	sea-orm-cli migrate up --database-url {{POSTGRES_URL}}


# Run pending migrations against postgres (docker)
[working-directory: 'server']
@migrate-up-server:
	sea-orm-cli migrate up --database-url {{DATABASE_URL}}

# Run pending migrations against sqlite
[working-directory: 'kernel']
@migrate-up-sqlite:
	sea-orm-cli migrate up --database-url {{SQLITE_URL}}

# Run pending migrations against mysql (docker)
[working-directory: 'kernel']
@migrate-up-mysql:
	sea-orm-cli migrate up --database-url {{MYSQL_URL}}

# Run pending migrations against all active databases
[working-directory: 'kernel']
@migrate-up:
	just migrate-up-postgres


# --- Entity generation ------------------------------------------------

[working-directory: 'kernel']
@generate-entities url:
	sea-orm-cli generate entity \
		--database-url {{url}} \
		--with-serde both \
		--model-extra-attributes 'serde(rename_all="camelCase")' \
		-o src/entities \
		--seaography

# Pull schema and generate entities for PostgreSQL using docker defaults (feature: postgres)
[working-directory: 'lunar']
@db-pull-postgres:
	sea-orm-cli generate entity \
		--database-url {{POSTGRES_URL}} \
		--with-serde both \
		--model-extra-attributes 'serde(rename_all="camelCase")' \
		--model-extra-derives 'ts_rs::TS' \
		--model-extra-attributes 'ts(export)' \
		--enum-extra-derives 'ts_rs::TS' \
		--enum-extra-attributes 'ts(export)' \
		-o src/entities 
	python3 ../scripts/fix-ts-exports.py src/entities
	cargo test --no-default-features  -- export_bindings
	npm run build
	


# Generate entities for all backends.
# Starts docker, waits for postgres, runs migrations (postgres + sqlite), then generates entities.
# Pass a custom sqlite URL or omit to use the default (sqlite://./almond.db?mode=rwc).
#   just db-pull
#   just db-pull sqlite://./custom.db
[working-directory: 'kernel']
db-pull:
	just db-up
	@echo "Waiting for PostgreSQL to be ready..."
	just migrate-up
	just db-pull-postgres
	just db-pull-server

[working-directory: 'server']
@generate-graphql-bindings url:
	sea-orm-cli generate entity \
		--database-url {{url}} \
		-o src/entities \
		--seaography


[working-directory:'kernel']
test-kernel:
    cargo test

db-pull-server:
	just migrate-up-server
	just generate-server-entities

	
[working-directory:'kernel']
gph-pull:
		graphql-client generate  --schema-path .graphql/sync_queue_schema.graphql .graphql/sync_queue_query.graphql --output-directory src


# --- WASM build & PGlite migration export ---------------------------------

# Export migration SQL to generated/pglite/migrations, then build the WASM bundle.
#   just build-wasm          # dev profile (matches pkg/ checked into the repo)
#   just build-wasm release  # release profile
[working-directory: 'lunar']
build-wasm profile='dev':
	cargo run --manifest-path migration/Cargo.toml --bin migration-export -- generated/pglite/migrations
	wasm-pack build --target web --{{profile}}
