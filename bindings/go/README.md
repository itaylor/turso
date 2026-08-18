<p align="center">
  <h1 align="center">Turso Database for Go</h1>
</p>

<p align="center">
  <a title="Go" target="_blank" href="https://pkg.go.dev/turso.tech/database/tursogo"><img src="https://pkg.go.dev/badge/turso.tech/database/tursogo.svg" alt="Go Reference"></a>
  <a title="MIT" target="_blank" href="https://github.com/tursodatabase/turso/blob/main/LICENSE.md"><img src="http://img.shields.io/badge/license-MIT-orange.svg?style=flat-square"></a>
</p>
<p align="center">
  <a title="Users Discord" target="_blank" href="https://tur.so/discord"><img alt="Chat with other users of Turso on Discord" src="https://img.shields.io/discord/933071162680958986?label=Discord&logo=Discord&style=social"></a>
</p>

---

## About

Turso Database runs in production today at multiple organizations. It has not yet reached 1.0, so as with any database we recommend keeping backups — see the [FAQ](https://github.com/tursodatabase/turso#faq) for where the project stands.

## Features

- **SQLite compatible:** SQLite query language and file format support ([status](https://github.com/tursodatabase/turso/blob/main/COMPAT.md)).
- **In-process**: No network overhead, runs directly in your Go process
- **Cross-platform**: Supports Linux, macOS, Windows
- **Remote partial sync**: Bootstrap from a remote database, pull remote changes, and push local changes when online &mdash; all while enjoying a fully operational database offline.
- **No CGO**: This driver uses the awesome [purego](https://github.com/ebitengine/purego) library to call C (in this case Rust with C ABI) functions from Go.

## Installation

```bash
go get turso.tech/database/tursogo
```

## Get Started

```go
package main

import (
	"database/sql"
	"fmt"
	"os"

	_ "turso.tech/database/tursogo"
)

func main() {
	conn, err := sql.Open("turso", ":memory:")
	if err != nil {
		fmt.Printf("Error: %v\n", err)
		os.Exit(1)
	}
	sql := "CREATE table go_turso (foo INTEGER, bar TEXT)"
	_, _ = conn.Exec(sql)

	sql = "INSERT INTO go_turso (foo, bar) values (?, ?)"
	stmt, _ := conn.Prepare(sql)
	defer stmt.Close()
	_, _ = stmt.Exec(42, "turso")
	rows, _ := conn.Query("SELECT * from go_turso")
	defer rows.Close()
	for rows.Next() {
		var a int
		var b string
		_ = rows.Scan(&a, &b)
		fmt.Printf("%d, %s\n", a, b) // 42, turso
	}
}
```

## User-defined functions

Register your own scalar and aggregate SQL functions on a connection, the same
way `sqlite3_create_function` does. Functions are per-connection, and
`database/sql` hands out pooled connections, so register on a pinned
`*sql.Conn` and run the queries that need the function on that same
connection.

```go
db, _ := sql.Open("turso", ":memory:")
db.SetMaxOpenConns(1)

conn, _ := db.Conn(context.Background())
defer conn.Close()

_ = conn.Raw(func(dc any) error {
	fns := dc.(turso.Functions)

	// Scalar function. Pass -1 for nArgs to make it variadic.
	return fns.CreateFunction("shout", 1, func(args []any) (any, error) {
		text, ok := args[0].(string)
		if !ok {
			return nil, fmt.Errorf("shout wants text, got %T", args[0])
		}
		return strings.ToUpper(text), nil
	}, &turso.FunctionOptions{Deterministic: true})
})

var loud string
_ = conn.QueryRowContext(ctx, "SELECT shout('hi')").Scan(&loud) // HI
```

Arguments arrive as `nil`, `int64`, `float64`, `string` or `[]byte`. A return
value may be `nil`, any signed or unsigned integer, `bool`, `float32`,
`float64`, `string` or `[]byte`; anything else fails the statement. Returning
an error (or panicking) fails the statement with that message, which surfaces
from `Exec` or from `rows.Err()`.

### Aggregates and window functions

`CreateAggregate` takes a constructor that is called once per group:

```go
type sum struct{ total int64 }

func (s *sum) Step(args []any) error { s.total += args[0].(int64); return nil }
func (s *sum) Final() (any, error)   { return s.total, nil }

// Optional: implementing these two makes the aggregate window-capable.
func (s *sum) Value() (any, error)      { return s.total, nil }
func (s *sum) Inverse(args []any) error { s.total -= args[0].(int64); return nil }

_ = conn.Raw(func(dc any) error {
	return dc.(turso.Functions).CreateAggregate("gosum", 1,
		func() turso.Aggregate { return &sum{} }, nil)
})
```

An aggregate whose instances implement `turso.WindowAggregate` (`Value` and
`Inverse` on top of `Step` and `Final`) can be used with `OVER (... ROWS
BETWEEN ...)`. Using one without them fails the query at run time with
"may not be used as a window function".

### Removing functions

`RemoveFunction(name)` unregisters the function. The underlying C ABI keys
removal by name alone, so it removes **every** arity registered under that
name, not one overload. Functions are also released when the connection
closes.

### Registering functions on a pool

The pinned-`*sql.Conn` approach above still works and is the simplest option
when only one connection needs the function. For a real pool
(`db.SetMaxOpenConns(n)` for `n > 1`), use `turso.NewConnector` with
`turso.WithFunctions` instead: the callback runs on every connection the pool
opens, right after it is created and before `database/sql` hands it out, so
the function is visible no matter which pooled connection a query lands on.

```go
connector, _ := turso.NewConnector(":memory:", turso.WithFunctions(func(fns turso.Functions) error {
	return fns.CreateFunction("shout", 1, func(args []any) (any, error) {
		text, ok := args[0].(string)
		if !ok {
			return nil, fmt.Errorf("shout wants text, got %T", args[0])
		}
		return strings.ToUpper(text), nil
	}, &turso.FunctionOptions{Deterministic: true})
}))

db := sql.OpenDB(connector)
db.SetMaxOpenConns(10) // safe: every connection gets `shout`

var loud string
_ = db.QueryRowContext(ctx, "SELECT shout('hi')").Scan(&loud) // HI
```

An error from the callback fails whichever call needed a new connection
(`Ping`, a query, ...). `WithFunctions` composes: pass it more than once to
run several registration steps in order. It only applies to connectors built
with `NewConnector` — plain `sql.Open("turso", dsn)` is unaffected and keeps
requiring the pinned-connection approach above.

## Sync Driver
Use a remote Turso database while working locally. You can bootstrap local state from the remote, pull remote changes, and push local commits.

Note: You need a Turso remote URL. See the Turso docs for provisioning and authentication.

```go
package main

import (
	"context"
	"fmt"
	"log"
	"os"

	turso "turso.tech/database/tursogo"
)

func main() {
	ctx := context.Background()

	// Connect a local database to a remote Turso database
	db, err := turso.NewTursoSyncDb(ctx, turso.TursoSyncDbConfig{
		Path:      ":memory:", // local db path (or a file path)
		RemoteUrl: "https://<db>.<region>.turso.io",
		AuthToken: "<authToken>",
	})
	if err != nil {
		fmt.Printf("Error: %v\n", err)
		os.Exit(1)
	}

	conn, err := db.Connect(ctx)
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	sql := "CREATE table go_turso (foo INTEGER, bar TEXT)"
	_, _ = conn.ExecContext(ctx, sql)

	sql = "INSERT INTO go_turso (foo, bar) values (?, ?)"
	stmt, _ := conn.PrepareContext(ctx, sql)
	defer stmt.Close()
	_, _ = stmt.ExecContext(ctx, 42, "turso")

	// Push local commits to remote
	_ = db.Push(ctx)

	// Pull new changes from remote into local
	_, _ = db.Pull(ctx)

	rows, _ := conn.QueryContext(ctx, "SELECT * from go_turso")
	defer rows.Close()
	for rows.Next() {
		var a int
		var b string
		_ = rows.Scan(&a, &b)
		fmt.Printf("%d, %s\n", a, b) // 42, turso
	}

	// Optional: inspect and manage sync state
	stats, err := db.Stats(ctx)
	if err != nil {
		log.Println("Stats unavailable:", err)
	} else {
		log.Println("Current revision:", stats.NetworkReceivedBytes)
	}

	_ = db.Checkpoint(ctx) // compact local WAL after many writes
}

```

## License

This project is licensed under the [MIT license](../../LICENSE.md).

## Support

- [GitHub Issues](https://github.com/tursodatabase/turso/issues)
- [Documentation](https://docs.turso.tech)
- [Discord Community](https://tur.so/discord)
