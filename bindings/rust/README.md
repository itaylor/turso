# turso

The next evolution of SQLite: A high-performance, SQLite-compatible database library for Rust

## Features

- **SQLite Compatible**: Similar interface to rusqlite with familiar API apart from using async Rust
- **High Performance**: Built with Rust for maximum speed and efficiency  
- **Async/Await Support**: Native async operations with tokio support
- **In-Process**: No network overhead, runs directly in your application
- **Cross-Platform**: Supports Linux, macOS, and Windows
- **Transaction Support**: Full ACID transactions with rollback support
- **Prepared Statements**: Optimized query execution with parameter binding
- **Cloud Sync**: Sync with Turso Cloud using `Builder::new_remote()` (optional `sync` feature)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
turso = "0.4.3"
tokio = { version = "1.0", features = ["full"] }
```

For cloud sync capabilities, enable the `sync` feature:

```toml
[dependencies]
turso = { version = "0.4.3", features = ["sync"] }
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

### In-Memory Database

```rust
use turso::Builder;

#[tokio::main]
async fn main() -> turso::Result<()> {
    // Create an in-memory database
    let db = Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create a table
    conn.execute(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)",
        ()
    ).await?;

    // Insert data
    conn.execute(
        "INSERT INTO users (name, email) VALUES (?1, ?2)",
        ["Alice", "alice@example.com"]
    ).await?;

    conn.execute(
        "INSERT INTO users (name, email) VALUES (?1, ?2)", 
        ["Bob", "bob@example.com"]
    ).await?;

    // Query data
    let mut rows = conn.query("SELECT * FROM users", ()).await?;
    
    while let Some(row) = rows.next().await? {
        let id = row.get_value(0)?;
        let name = row.get_value(1)?;
        let email = row.get_value(2)?;
        println!("User: {} - {} ({})", 
            id.as_integer().unwrap_or(&0), 
            name.as_text().unwrap_or(&"".to_string()), 
            email.as_text().unwrap_or(&"".to_string())
        );
    }

    Ok(())
}
```

### File-Based Database

```rust
use turso::Builder;

#[tokio::main] 
async fn main() -> turso::Result<()> {
    // Create or open a database file
    let db = Builder::new_local("my-database.db").build().await?;
    let conn = db.connect()?;

    // Create a table
    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"#,
        ()
    ).await?;

    // Insert a post
    let rows_affected = conn.execute(
        "INSERT INTO posts (title, content) VALUES (?1, ?2)",
        ["Hello World", "This is my first blog post!"]
    ).await?;

    println!("Inserted {} rows", rows_affected);

    Ok(())
}
```

### Synced Database

Sync your local database with Turso Cloud:

```rust
use turso::sync::Builder;

#[tokio::main]
async fn main() -> turso::Result<()> {
    // Create a synced database
    let db = Builder::new_remote("local.db")
        .with_remote_url("libsql://your-database.turso.io")
        .with_auth_token("your-token")
        .build()
        .await?;

    let conn = db.connect().await?;

    // Create a table and insert data
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, content TEXT)",
        ()
    ).await?;

    conn.execute(
        "INSERT INTO notes (content) VALUES (?1)",
        ["My first synced note"]
    ).await?;

    // Push local changes to remote
    db.push().await?;

    // Pull remote changes to local
    db.pull().await?;

    Ok(())
}
```

## API Reference

### Builder

Create a new database:

```rust
let db = Builder::new_local(":memory:").build().await?;
let db = Builder::new_local("data.db").build().await?;
```

### Connection

Execute queries and statements:

```rust
// Execute SQL directly
let rows_affected = conn.execute("INSERT INTO users (name) VALUES (?1)", ["Alice"]).await?;

// Query for multiple rows
let mut rows = conn.query("SELECT * FROM users WHERE age > ?1", [18]).await?;

// Prepare statements for reuse
let mut stmt = conn.prepare("SELECT * FROM users WHERE id = ?1").await?;
let mut rows = stmt.query([42]).await?;

// Execute prepared statements
let rows_affected = stmt.execute(["Alice"]).await?;
```

### Working with Results

```rust
use futures_util::TryStreamExt;

let mut rows = conn.query("SELECT name, email FROM users", ()).await?;

while let Some(row) = rows.try_next().await? {
    let name = row.get_value(0)?.as_text().unwrap_or(&"".to_string());
    let email = row.get_value(1)?.as_text().unwrap_or(&"".to_string());
    println!("{}: {}", name, email);
}
```

### Sync API Reference

#### sync::Builder

Create a synced database that synchronizes with Turso Cloud:

```rust
use turso::sync::Builder;

let db = Builder::new_remote("local.db")       // Local database path (or ":memory:")
    .with_remote_url("libsql://db.turso.io")   // Remote URL (https://, http://, or libsql://)
    .with_auth_token("your-token")              // Authorization token
    .bootstrap_if_empty(true)                   // Download schema on first sync (default: true)
    .with_remote_encryption("base64-encoded-key", RemoteEncryptionCipher::Aes256Gcm) // Optional remote encryption
    .build()
    .await?;
```

#### sync::Database

Operations for synced databases:

```rust
// Push local changes to remote
db.push().await?;

// Pull remote changes (returns true if changes were applied)
let had_changes = db.pull().await?;

// Force WAL checkpoint
db.checkpoint().await?;

// Get sync statistics
let stats = db.stats().await?;
println!("Received: {} bytes", stats.network_received_bytes);
println!("Sent: {} bytes", stats.network_sent_bytes);
println!("WAL size: {} bytes", stats.main_wal_size);
```

### User-Defined Functions

Register scalar and aggregate (optionally window-capable) functions on a
connection, the way `sqlite3_create_function` does. Registration is
per-connection, and a function shadows a built-in of the same name and
argument count.

```rust
use turso::udf::FunctionFlags;
use turso::{Builder, Value, ValueRef};

let db = Builder::new_local(":memory:").build().await?;
let conn = db.connect()?;

conn.create_scalar_function(
    "double",
    1,
    FunctionFlags::DETERMINISTIC,
    |args: &[ValueRef<'_>]| {
        let n = args[0].as_integer().copied().unwrap_or(0);
        Ok(Value::Integer(n * 2))
    },
)?;

let mut rows = conn.query("SELECT double(21)", ()).await?;
let row = rows.next().await?.unwrap();
assert_eq!(row.get_value(0)?, Value::Integer(42));
```

An aggregate is any type implementing `turso::udf::AggregateFunction` /
`AggregateState`; implement `value`/`inverse` and return `true` from
`supports_window()` to let it also run with `OVER (...)`:

```rust
use turso::udf::{AggregateFunction, AggregateState, FunctionFlags};

struct Sum;
struct SumState(i64);

impl AggregateFunction for Sum {
    fn init(&self) -> turso::Result<Box<dyn AggregateState>> {
        Ok(Box::new(SumState(0)))
    }
}

impl AggregateState for SumState {
    fn step(&mut self, args: &[turso::ValueRef<'_>]) -> turso::Result<()> {
        self.0 += args[0].as_integer().copied().unwrap_or(0);
        Ok(())
    }

    fn finalize(self: Box<Self>) -> turso::Result<turso::Value> {
        Ok(turso::Value::Integer(self.0))
    }
}

conn.create_aggregate_function("my_sum", 1, FunctionFlags::empty(), Sum)?;
```

See the `turso::udf` module docs for the full API, including `remove_function`
and the flags (`DETERMINISTIC`, `DIRECTONLY`, `INNOCUOUS`).

#### Database-level registration

`Database` has the same `create_scalar_function` / `create_aggregate_function`
/ `remove_function` methods, but registering there seeds every connection
opened with `connect()` *afterwards* instead of just one connection. This has
no SQLite equivalent — it's a Turso convenience for cases where many
connections need the same function and registering on each one individually
would be repetitive:

```rust
let db = Builder::new_local(":memory:").build().await?;

db.create_scalar_function(
    "double",
    1,
    FunctionFlags::DETERMINISTIC,
    |args: &[ValueRef<'_>]| {
        let n = args[0].as_integer().copied().unwrap_or(0);
        Ok(Value::Integer(n * 2))
    },
)?;

// Every connection opened from here on sees `double`.
let conn = db.connect()?;
let mut rows = conn.query("SELECT double(21)", ()).await?;
```

A connection opened *before* the call, or already open, does not see the
function — it already has its own copy of the function table. A
connection-level registration of the same name and argument count shadows
the database-level one, but only on that connection.

## License

MIT

## Support

- [GitHub Issues](https://github.com/tursodatabase/turso/issues)
- [Discord Community](https://discord.gg/turso)
