# Turso JDBC Driver

The Turso JDBC driver is a library for accessing and creating Turso database files using Java.

## Project Status

The project is actively developed. Feel free to open issues and contribute.

To view related works, visit this [issue](https://github.com/tursodatabase/turso/issues/615).

## How to use

Currently, we have not published to the maven central. Instead, you can locally build the jar and deploy it to
maven local to use it.

### Build jar and publish to maven local

```shell
$ cd bindings/java

# Please select the appropriate target platform, currently supports `macos_x86`, `macos_arm64`, `windows` and `linux_x86`
$ make macos_x86

# deploy to maven local
$ make publish_local
```

Now you can use the dependency as follows:

```kotlin
dependencies {
    implementation("tech.turso:turso:0.0.1-SNAPSHOT")
}
```

## User-Defined Functions

A connection can register its own scalar, aggregate and window functions,
the way `sqlite3_create_function` does. Functions are per-connection and
disappear when the connection closes; a function registered under the same
name and argument count as a built-in shadows it.

```java
// createFunction/createAggregate/createWindowFunction/removeFunction are not
// part of java.sql.Connection, so cast the java.sql.Connection DriverManager
// gives you back to JDBC4Connection to reach them.
JDBC4Connection connection =
    (JDBC4Connection) DriverManager.getConnection("jdbc:turso:" + path);

// Scalar: a lambda works because ScalarFunction is a functional interface
connection.createFunction("plus_one", 1, TursoFunction.DETERMINISTIC,
    args -> (Long) args[0] + 1);

// Aggregate: the accumulator is returned from step(), so an immutable type works
connection.createAggregate("sum_lengths", 1, 0, new Aggregate<Long>() {
  public Long init() {
    return 0L;
  }

  public Long step(Long total, Object[] args) {
    return total + ((String) args[0]).length();
  }

  public Object result(Long total) {
    return total;
  }
});

// Window: extends Aggregate with value()/inverse(), so it also works with OVER (...)
connection.createWindowFunction("running_sum", 1, 0, new WindowFunction<Long>() {
  public Long init() {
    return 0L;
  }

  public Long step(Long total, Object[] args) {
    return total + (Long) args[0];
  }

  public Object result(Long total) {
    return total;
  }

  public Object value(Long total) {
    return total;
  }

  public Long inverse(Long total, Object[] args) {
    return total - (Long) args[0];
  }
});

connection.removeFunction("plus_one", 1);
```

Arguments arrive as `null`, `Long`, `Double`, `String` or `byte[]`; the
return value may be any boxed primitive, `String` or `byte[]`. Flags —
`TursoFunction.DETERMINISTIC`, `DIRECTONLY`, `INNOCUOUS` — may be combined
with `|`, or pass `0` for none. See
[`ScalarFunction`](src/main/java/tech/turso/core/ScalarFunction.java),
[`Aggregate`](src/main/java/tech/turso/core/Aggregate.java),
[`WindowFunction`](src/main/java/tech/turso/core/WindowFunction.java) and
[`TursoConnection`](src/main/java/tech/turso/core/TursoConnection.java) for
the full API.

## Code style

- Favor composition over inheritance. For example, `JDBC4Connection` doesn't implement `TursoConnection`. Instead,
  it includes `TursoConnection` as a field. This approach allows us to preserve the characteristics of Turso using
  `TursoConnection` easily while maintaining interoperability with the Java world using `JDBC4Connection`.
