# class Database

The `Database` class represents a connection that can prepare and execute SQL statements.

## Methods

### new Database(path, [options]) ⇒ Database

Creates a new database connection.

| Param   | Type                | Description               |
| ------- | ------------------- | ------------------------- |
| path    | <code>string</code> | Path to the database file |
| options | <code>object</code> | Options.                  |

The `path` parameter points to the SQLite database file to open. If the file pointed to by `path` does not exists, it will be created.
To open an in-memory database, please pass `:memory:` as the `path` parameter.

Supported `options` fields include:

- `timeout`: busy timeout in milliseconds
- `defaultQueryTimeout`: default maximum query execution time in milliseconds before interruption

Per-query timeout override is available via `queryOptions`, for example:

- `db.exec("SELECT 1", { queryTimeout: 100 })`
- `stmt.get(undefined, { queryTimeout: 100 })`

The function returns a `Database` object.

### prepare(sql) ⇒ Statement

Prepares a SQL statement for execution.

| Param | Type                | Description                          |
| ----- | ------------------- | ------------------------------------ |
| sql   | <code>string</code> | The SQL statement string to prepare. |

The function returns a `Statement` object.

### run(sql[, ...bindParameters][, queryOptions]) ⇒ object

Convenience wrapper that prepares `sql` and executes `Statement.run`. Returns the same info object as `Statement.run` (`changes` and `lastInsertRowid`). The internally prepared statement is finalized before the call returns.

| Param          | Type                | Description                                                          |
| -------------- | ------------------- | -------------------------------------------------------------------- |
| sql            | <code>string</code> | The SQL statement string.                                            |
| bindParameters | <code>any</code>    | Optional positional or named bind parameters.                        |
| queryOptions   | <code>object</code> | Optional per-query overrides (for example, `{ queryTimeout: 100 }`). |

**Note:** This is an extension in libSQL and not available in `better-sqlite3`.

### get(sql[, ...bindParameters][, queryOptions]) ⇒ row

Convenience wrapper that prepares `sql` and executes `Statement.get`. Returns the first row, or `undefined` if no row matched. The internally prepared statement is finalized before the call returns.

| Param          | Type                | Description                                                          |
| -------------- | ------------------- | -------------------------------------------------------------------- |
| sql            | <code>string</code> | The SQL statement string.                                            |
| bindParameters | <code>any</code>    | Optional positional or named bind parameters.                        |
| queryOptions   | <code>object</code> | Optional per-query overrides (for example, `{ queryTimeout: 100 }`). |

**Note:** This is an extension in libSQL and not available in `better-sqlite3`.

### all(sql[, ...bindParameters][, queryOptions]) ⇒ array of rows

Convenience wrapper that prepares `sql` and executes `Statement.all`. Returns all matching rows as an array. The internally prepared statement is finalized before the call returns.

| Param          | Type                | Description                                                          |
| -------------- | ------------------- | -------------------------------------------------------------------- |
| sql            | <code>string</code> | The SQL statement string.                                            |
| bindParameters | <code>any</code>    | Optional positional or named bind parameters.                        |
| queryOptions   | <code>object</code> | Optional per-query overrides (for example, `{ queryTimeout: 100 }`). |

**Note:** This is an extension in libSQL and not available in `better-sqlite3`.

### iterate(sql[, ...bindParameters][, queryOptions]) ⇒ async iterator

Convenience wrapper that prepares `sql` and executes `Statement.iterate`. Returns an async iterator over the resulting rows. The internally prepared statement is finalized when the iterator is exhausted or closed.

| Param          | Type                | Description                                                          |
| -------------- | ------------------- | -------------------------------------------------------------------- |
| sql            | <code>string</code> | The SQL statement string.                                            |
| bindParameters | <code>any</code>    | Optional positional or named bind parameters.                        |
| queryOptions   | <code>object</code> | Optional per-query overrides (for example, `{ queryTimeout: 100 }`). |

**Note:** This is an extension in libSQL and not available in `better-sqlite3`.

### transaction(function) ⇒ function

Returns a function that runs the given function in a transaction.

| Param    | Type                  | Description                           |
| -------- | --------------------- | ------------------------------------- |
| function | <code>function</code> | The function to run in a transaction. |

### pragma(string, [options]) ⇒ results

Executes the given PRAGMA and returns its results.

| Param    | Type                  | Description           |
| -------- | --------------------- | ----------------------|
| source   | <code>string</code>   | Pragma to be executed |
| options  | <code>object</code>   | Options.              |

Most PRAGMA return a single value, the `simple: boolean` option is provided to return the first column of the first row.

```js
db.pragma('cache_size = 32000');
console.log(db.pragma('cache_size', { simple: true })); // => 32000
```

### backup(destination, [options]) ⇒ promise

This function is currently not supported.

### serialize([options]) ⇒ Buffer

This function is currently not supported.

### function(name, [options], function) ⇒ this

Registers a scalar user-defined function, callable from SQL on this
connection. Returns the database, so calls can be chained. The callback runs
while a statement is stepping, so it must be synchronous.

| Param    | Type                  | Description                                                                       |
| -------- | --------------------- | --------------------------------------------------------------------------------- |
| name     | <code>string</code>   | The name SQL calls the function by.                                               |
| options  | <code>object</code>   | Optional. See below.                                                              |
| function | <code>function</code> | The implementation.                                                               |

| Option        | Default              | Description                                                                                 |
| ------------- | -------------------- | ------------------------------------------------------------------------------------------- |
| deterministic | `false`              | The same arguments always produce the same result, so the engine may call it once and reuse the answer. |
| varargs       | `false`              | Accept any number of arguments. Without it the arity is `function.length`, and a call with a different number of arguments fails with `wrong number of arguments`. |
| directOnly    | `false`              | Mark the function as callable only from top-level SQL, never from a trigger, view, CHECK constraint, DEFAULT, generated column or index expression; calling it from there fails with `unsafe use of X()`. |
| innocuous     | `false`              | Mark the function as safe to run from schema SQL (a view, trigger, CHECK constraint, ...) even when `PRAGMA trusted_schema=OFF`; without it such a call fails with `unsafe use of X()` once trusted_schema is off. |
| safeIntegers  | database default     | Pass INTEGER arguments as BigInt instead of Number. Follows `defaultSafeIntegers()` when omitted. |

```js
db.function('add2', (a, b) => a + b);
db.prepare('SELECT add2(2, 3) AS v').get(); // => { v: 5 }

db.function('sum_all', { varargs: true }, (...args) => args.reduce((a, b) => a + b, 0));
```

Arguments arrive as `null`, Number (or BigInt with `safeIntegers`), string or
`Uint8Array`. The return value may be `null`/`undefined` (SQL NULL), a number,
a BigInt, a string, a boolean (0 or 1) or a `Buffer`/`Uint8Array`; anything
else fails the statement with a `TypeError`. An exception thrown by the
callback aborts the statement and is rethrown to whoever ran it, unchanged.

A user-defined function may run SQL of its own with `db.prepare(...)`, but not
through the statement that invoked it — that statement is mid-step and reports
`statement is already running`. A function registered with the same name and
argument count as a built-in shadows it.

### aggregate(name, options) ⇒ this

Registers an aggregate user-defined function. Returns the database.

| Option        | Default              | Description                                                                                  |
| ------------- | -------------------- | ---------------------------------------------------------------------------------------------- |
| start         | `null`               | The initial value of the accumulator, or a function returning a fresh one for every group.   |
| step          | *required*           | `(total, ...args)`. Returns the new accumulator, or `undefined` to keep the current one.     |
| inverse       | none                 | `(total, ...args)`. Removes a row that left the window frame. Providing it makes the aggregate usable as a window function. |
| result        | none                 | `(total)`. Turns the final accumulator into the value SQL sees. Defaults to the accumulator itself. |
| deterministic, varargs, directOnly, innocuous, safeIntegers | | As for `function()`. The arity is `step.length - 1`, since `step` also takes the accumulator. |

```js
db.aggregate('mysum', {
  start: 0,
  step: (total, x) => total + x,
  inverse: (total, x) => total - x,
});

db.prepare('SELECT mysum(x) AS v FROM t').get();
db.prepare(
  'SELECT x, mysum(x) OVER (ORDER BY x ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) AS v FROM t'
).all();
```

The accumulator can be any JavaScript value, including an object or array.
Without an `inverse` callback the aggregate cannot be used with `OVER`, and
such a query fails to compile with `X() may not be used as a window function`.

### table(name, definition) ⇒ this

This function is currently not supported.

### loadExtension(path, [entryPoint]) ⇒ this

Loads a SQLite3 extension.

| Param | Type                | Description                             |
| ----- | ------------------- | --------------------------------------- |
| path  | <code>string</code> | The path to the extension to be loaded. |

### exec(sql) ⇒ this

Executes a SQL statement.

| Param | Type                | Description                          |
| ----- | ------------------- | ------------------------------------ |
| sql   | <code>string</code> | The SQL statement string to execute. |

This can execute strings that contain multiple SQL statements.

### interrupt() ⇒ this

Cancel ongoing operations and make them return at earliest opportunity.

**Note:** This is an extension in libSQL and not available in `better-sqlite3`.

This function is currently not supported.

### close() ⇒ this

Closes the database connection.

# class Statement

## Methods

### run([...bindParameters]) ⇒ object

Executes the SQL statement and (currently) returns an array with results.

**Note:** It should return an info object.

| Param          | Type                          | Description                                      |
| -------------- | ----------------------------- | ------------------------------------------------ |
| bindParameters | <code>array of objects</code> | The bind parameters for executing the statement. |

The returned info object contains two properties: `changes` that describes the number of modified rows and `info.lastInsertRowid` that represents the `rowid` of the last inserted row.

This function is currently not supported.

### get([...bindParameters]) ⇒ row

Executes the SQL statement and returns the first row.

| Param          | Type                          | Description                                      |
| -------------- | ----------------------------- | ------------------------------------------------ |
| bindParameters | <code>array of objects</code> | The bind parameters for executing the statement. |

### all([...bindParameters]) ⇒ array of rows

Executes the SQL statement and returns an array of the resulting rows.

| Param          | Type                          | Description                                      |
| -------------- | ----------------------------- | ------------------------------------------------ |
| bindParameters | <code>array of objects</code> | The bind parameters for executing the statement. |

### iterate([...bindParameters]) ⇒ iterator

Executes the SQL statement and returns an iterator to the resulting rows.

| Param          | Type                          | Description                                      |
| -------------- | ----------------------------- | ------------------------------------------------ |
| bindParameters | <code>array of objects</code> | The bind parameters for executing the statement. |

### pluck([toggleState]) ⇒ this

Makes the prepared statement only return the value of the first column of any rows that it retrieves.

| Param     | Type                 | Description                                                                            |
| --------- | -------------------- | -------------------------------------------------------------------------------------- |
| pluckMode | <code>boolean</code> | Enable of disable pluck mode. If you don't pass the paramenter, pluck mode is enabled. |

```js
stmt.pluck(); // plucking ON
stmt.pluck(true); // plucking ON
stmt.pluck(false); // plucking OFF
```

> NOTE: When plucking is turned on, raw mode is turned off (they are mutually exclusive options).

### expand([toggleState]) ⇒ this

This function is currently not supported.

### raw([rawMode]) ⇒ this

Makes the prepared statement return rows as arrays instead of objects.

| Param   | Type                 | Description                                                                       |
| ------- | -------------------- | --------------------------------------------------------------------------------- |
| rawMode | <code>boolean</code> | Enable or disable raw mode. If you don't pass the parameter, raw mode is enabled. |

This function enables or disables raw mode. Prepared statements return objects by default, but if raw mode is enabled, the functions return arrays instead.

```js
stmt.raw(); // raw mode ON
stmt.raw(true); // raw mode ON
stmt.raw(false); // raw mode OFF
```

> NOTE: When raw mode is turned on, plucking is turned off (they are mutually exclusive options).

### columns() ⇒ array of objects

Returns the columns in the result set returned by this prepared statement.

This function is currently not supported.

### bind([...bindParameters]) ⇒ this

| Param          | Type                          | Description                                      |
| -------------- | ----------------------------- | ------------------------------------------------ |
| bindParameters | <code>array of objects</code> | The bind parameters for executing the statement. |

Binds **permanently** the given parameters to the statement. After a statement's parameters are bound this way, you may no longer provide it with execution-specific (temporary) bound parameters.
