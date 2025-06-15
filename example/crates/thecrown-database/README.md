# Diesel
Diesel is an ORM library for Rust.
It supports MySQL, Postgres and SQLite and can manage migrations.

Diesel comes with a CLI tool to manage migrations. A configuration file (`diesel.toml`) may be placed
in the cargo project.
```toml
[migrations_directory]
dir = "migrations" # folder containing the migrations
```

A table with the name `__diesel_schema_migrations` is automatically created on the database to keep
track of all the migrations run.

## Installing diesel-cli
```bash
sudo pacman -S diesel-cli
```

## Creating a migration
```bash
diesel migration generate <name>
```
This command will generate a migration in the migration folder with the current timestamp. The files
`up.sql` and `down.sql` created.

## Executing migrations
```bash
diesel migration <run|redo|revert>
```
This command will run, redo or revert the migration on the database. The database service address must
be passed using the `--database-url` parameter or by setting the `DATBASE_URL` enviroment variable.

## Generating schema file
```bash
diesel print-schema > src/schema.rs
```

## Running migrations automatically
The following functions executes pending migrations
```rust
pub fn run_embedded_migrations(&self) {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

    self.get_connection().run_pending_migrations(MIGRATIONS).unwrap();
}
```
you can call it right after the connection