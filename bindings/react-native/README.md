# Turso Sync React Native SDK

React Native bindings for Turso embedded replicas - sync your local SQLite database with Turso cloud.

## Installation

```bash
npm install @tursodatabase/sync-react-native
```

### iOS

```bash
cd ios && pod install
```

### Android

Requires `minSdkVersion` 21+ in `android/build.gradle`.

## Quick Start

```typescript
import { Database, getDbPath } from '@tursodatabase/sync-react-native';

// Get platform-specific writable path
const dbPath = getDbPath('myapp.db');

// Create database with sync
const db = new Database({
  path: dbPath,
  url: 'libsql://your-db.turso.io',
  authToken: 'your-auth-token',
});

// Connect (bootstraps from remote if empty)
await db.connect();

// Query local replica (fast)
const users = await db.all('SELECT * FROM users');

// Make local changes
await db.run('INSERT INTO users (name) VALUES (?)', ['Alice']);

// Sync with remote
await db.push();  // Push local changes
await db.pull();  // Pull remote changes

// Close when done
await db.close();
```

## Local-Only Database

```typescript
const db = new Database({ path: getDbPath('local.db') });
await db.connect();

await db.exec('CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)');
await db.run('INSERT INTO users (name) VALUES (?)', ['Bob']);
const user = await db.get('SELECT * FROM users WHERE id = ?', [1]);

await db.close();
```

## Encrypted Remote Database

```typescript
const db = new Database({
  path: getDbPath('encrypted.db'),
  url: 'libsql://your-db.turso.io',
  authToken: 'your-auth-token',
  remoteEncryption: {
    cipher: 'aes256gcm',
    key: 'base64-encoded-key',
  },
});
```

## API

### Database Methods

| Method | Description |
|--------|-------------|
| `connect()` | Open/bootstrap the database |
| `exec(sql)` | Execute SQL (no results) |
| `run(sql, params?)` | Execute SQL, return `{ changes, lastInsertRowid }` |
| `get(sql, params?)` | Query single row |
| `all(sql, params?)` | Query all rows |
| `prepare(sql)` | Create prepared statement |
| `function(name, [options], fn)` | Register a scalar user-defined function |
| `aggregate(name, options)` | Register an aggregate (and window) user-defined function |
| `removeFunction(name)` | Remove every registration of `name` |
| `close()` | Close database |

### Sync Methods (when `url` is provided)

| Method | Description |
|--------|-------------|
| `push()` | Push local changes to remote |
| `pull()` | Pull remote changes to local |
| `sync()` | Push then pull |
| `stats()` | Get sync statistics |

### User-Defined Functions

`function()` and `aggregate()` take the same options as better-sqlite3.

```typescript
// Scalar: fn.length decides how many arguments SQL must pass
db.function('add_two', (a: number, b: number) => a + b);
await db.get('SELECT add_two(2, 3) AS sum'); // { sum: 5 }

// Options come before the implementation
db.function('shout', { deterministic: true }, (text: string) => `${text}!`);

// varargs accepts any number of arguments
db.function('join_all', { varargs: true }, (...parts: string[]) => parts.join('-'));

// Aggregate
db.aggregate('total_len', {
  start: 0,
  step: (total: number, value: string) => total + value.length,
});
await db.get('SELECT total_len(name) AS n FROM users');

// Adding `inverse` makes the aggregate usable as a window function
db.aggregate('running_sum', {
  start: 0,
  step: (total: number, value: number) => total + value,
  inverse: (total: number, value: number) => total - value,
  result: (total: number) => total,
});
await db.all(
  'SELECT running_sum(x) OVER (ORDER BY x ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) AS s FROM t'
);

// Throwing fails the statement with the thrown message
db.function('strict_abs', (value: number) => {
  if (typeof value !== 'number') throw new TypeError('expected a number');
  return Math.abs(value);
});

db.removeFunction('add_two');
```

The callbacks run **synchronously on the JavaScript thread**, nested inside the
`step()` that reached them:

- They must be synchronous. An `async` callback returns a Promise, which SQL has
  no type for, and the statement fails.
- They cannot query the database: every query method here is `async` and waits
  on the lock the running statement already holds, so the Promise a callback
  would return never resolves.
- Values map as they do for column values: `null`, number, string and
  `ArrayBuffer` (a typed array is accepted too and copied as a blob).
  `undefined` means SQL NULL, and `true`/`false` become 1/0. INTEGER values
  always arrive as JavaScript numbers — there is no `safeIntegers` option, and
  passing one throws.
- `deterministic`, `directOnly` and `innocuous` work for scalars and
  aggregates alike; they are the same flags SQLite's `SQLITE_DETERMINISTIC`,
  `SQLITE_DIRECTONLY` and `SQLITE_INNOCUOUS` set.

### Transactions

```typescript
await db.transaction(async () => {
  await db.run('INSERT INTO users (name) VALUES (?)', ['Alice']);
  await db.run('INSERT INTO users (name) VALUES (?)', ['Bob']);
  // Commits on success, rolls back on error
});
```
## License

This project is licensed under the [MIT license](https://github.com/tursodatabase/turso/blob/main/LICENSE.md).

## Links

- [Turso Documentation](https://docs.turso.tech)
- [GitHub](https://github.com/tursodatabase/turso)
- [npm](https://www.npmjs.com/package/@tursodatabase/sync-react-native)
