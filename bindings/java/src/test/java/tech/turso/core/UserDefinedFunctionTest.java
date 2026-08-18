package tech.turso.core;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertIterableEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.sql.ResultSet;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Properties;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import tech.turso.TestUtils;
import tech.turso.jdbc4.JDBC4Connection;

class UserDefinedFunctionTest {

  private JDBC4Connection connection;

  @BeforeEach
  void setUp() throws Exception {
    String filePath = TestUtils.createTempFile();
    String url = "jdbc:turso:" + filePath;
    connection = new JDBC4Connection(url, filePath, new Properties());
  }

  @Test
  void scalar_function_receives_its_arguments_and_returns_a_value() throws Exception {
    connection.createFunction("add_two", 2, 0, args -> (Long) args[0] + (Long) args[1]);

    assertEquals(7L, queryOneValue("SELECT add_two(3, 4)"));
  }

  @Test
  void scalar_function_registered_with_minus_one_takes_any_number_of_arguments() throws Exception {
    connection.createFunction("count_args", -1, 0, args -> (long) args.length);

    assertEquals(0L, queryOneValue("SELECT count_args()"));
    assertEquals(1L, queryOneValue("SELECT count_args('a')"));
    assertEquals(3L, queryOneValue("SELECT count_args(1, 'two', NULL)"));
  }

  @Test
  void calling_a_function_with_the_wrong_number_of_arguments_fails() throws Exception {
    connection.createFunction("needs_one", 1, 0, args -> args[0]);

    SQLException exception =
        assertThrows(SQLException.class, () -> queryOneValue("SELECT needs_one(1, 2)"));
    assertTrue(
        exception.getMessage().contains("wrong number of arguments"), exception.getMessage());
  }

  @Test
  void arguments_arrive_as_the_java_type_matching_their_sql_type() throws Exception {
    connection.createFunction(
        "describe",
        1,
        0,
        args -> {
          Object arg = args[0];
          if (arg == null) {
            return "null";
          }
          if (arg instanceof byte[]) {
            return "byte[]:" + Arrays.toString((byte[]) arg);
          }
          return arg.getClass().getSimpleName() + ":" + arg;
        });

    assertEquals("null", queryOneValue("SELECT describe(NULL)"));
    assertEquals("Long:7", queryOneValue("SELECT describe(7)"));
    assertEquals("Double:1.5", queryOneValue("SELECT describe(1.5)"));
    assertEquals("String:hi", queryOneValue("SELECT describe('hi')"));
    assertEquals("byte[]:[1, 2, 3]", queryOneValue("SELECT describe(x'010203')"));
  }

  @Test
  void every_supported_return_type_reaches_sql() throws Exception {
    connection.createFunction("ret_null", 0, 0, args -> null);
    connection.createFunction("ret_long", 0, 0, args -> 9000000000L);
    connection.createFunction("ret_integer", 0, 0, args -> Integer.valueOf(42));
    connection.createFunction("ret_short", 0, 0, args -> Short.valueOf((short) 7));
    connection.createFunction("ret_byte", 0, 0, args -> Byte.valueOf((byte) 3));
    connection.createFunction("ret_boolean", 0, 0, args -> Boolean.TRUE);
    connection.createFunction("ret_double", 0, 0, args -> Double.valueOf(2.5));
    connection.createFunction("ret_float", 0, 0, args -> Float.valueOf(1.5f));
    connection.createFunction("ret_string", 0, 0, args -> "hello");
    connection.createFunction("ret_blob", 0, 0, args -> new byte[] {1, 2, 3});

    assertNull(queryOneValue("SELECT ret_null()"));
    assertEquals(9000000000L, queryOneValue("SELECT ret_long()"));
    assertEquals(42L, queryOneValue("SELECT ret_integer()"));
    assertEquals(7L, queryOneValue("SELECT ret_short()"));
    assertEquals(3L, queryOneValue("SELECT ret_byte()"));
    assertEquals(1L, queryOneValue("SELECT ret_boolean()"));
    assertEquals(2.5, queryOneValue("SELECT ret_double()"));
    assertEquals(1.5, queryOneValue("SELECT ret_float()"));
    assertEquals("hello", queryOneValue("SELECT ret_string()"));
    assertArrayEquals(new byte[] {1, 2, 3}, (byte[]) queryOneValue("SELECT ret_blob()"));
  }

  @Test
  void returning_a_type_sql_has_no_room_for_fails_the_statement() throws Exception {
    connection.createFunction("ret_list", 0, 0, args -> new ArrayList<String>());

    SQLException exception =
        assertThrows(SQLException.class, () -> queryOneValue("SELECT ret_list()"));
    assertTrue(exception.getMessage().contains("unsupported return type"), exception.getMessage());
  }

  @Test
  void an_exception_from_user_code_fails_the_statement_with_its_message() throws Exception {
    connection.createFunction(
        "boom",
        0,
        0,
        args -> {
          throw new IllegalStateException("kaboom");
        });

    SQLException exception = assertThrows(SQLException.class, () -> queryOneValue("SELECT boom()"));
    assertTrue(
        exception.getMessage().contains("user-defined function raised exception: kaboom"),
        exception.getMessage());
  }

  @Test
  void a_deterministic_function_of_constants_is_called_only_once() throws Exception {
    execute("CREATE TABLE t (v INTEGER)");
    execute("INSERT INTO t VALUES (1), (2), (3)");

    AtomicInteger calls = new AtomicInteger();
    connection.createFunction(
        "det_double",
        1,
        TursoFunction.DETERMINISTIC,
        args -> {
          calls.incrementAndGet();
          return (Long) args[0] * 2;
        });

    assertIterableEquals(
        Arrays.asList(10L, 10L, 10L), queryFirstColumn("SELECT det_double(5) FROM t"));
    assertEquals(1, calls.get());
  }

  @Test
  void aggregate_folds_each_group_on_its_own() throws Exception {
    execute("CREATE TABLE sales (region TEXT, amount INTEGER)");
    execute("INSERT INTO sales VALUES ('east', 1), ('west', 10), ('east', 2), ('west', 20)");

    connection.createAggregate("my_sum", 1, 0, new SumAggregate());

    assertIterableEquals(
        Arrays.asList(3L, 30L),
        queryFirstColumn("SELECT my_sum(amount) FROM sales GROUP BY region ORDER BY region"));
  }

  @Test
  void aggregate_with_no_rows_still_produces_a_value() throws Exception {
    execute("CREATE TABLE empty_table (amount INTEGER)");
    connection.createAggregate("my_sum", 1, 0, new SumAggregate());

    assertEquals(0L, queryOneValue("SELECT my_sum(amount) FROM empty_table"));
  }

  @Test
  void window_function_runs_over_a_moving_frame() throws Exception {
    execute("CREATE TABLE readings (id INTEGER, v INTEGER)");
    execute("INSERT INTO readings VALUES (1, 1), (2, 2), (3, 3), (4, 4)");

    connection.createWindowFunction("win_sum", 1, 0, new SumWindowFunction());

    assertIterableEquals(
        Arrays.asList(1L, 3L, 5L, 7L),
        queryFirstColumn(
            "SELECT win_sum(v) OVER (ORDER BY id ROWS BETWEEN 1 PRECEDING AND CURRENT ROW)"
                + " FROM readings ORDER BY id"));
  }

  @Test
  void window_function_also_works_as_a_plain_aggregate() throws Exception {
    execute("CREATE TABLE readings (v INTEGER)");
    execute("INSERT INTO readings VALUES (1), (2), (3)");

    connection.createWindowFunction("win_sum", 1, 0, new SumWindowFunction());

    assertEquals(6L, queryOneValue("SELECT win_sum(v) FROM readings"));
  }

  @Test
  void a_plain_aggregate_used_with_a_moving_frame_is_rejected() throws Exception {
    execute("CREATE TABLE readings (id INTEGER, v INTEGER)");
    execute("INSERT INTO readings VALUES (1, 1), (2, 2)");

    connection.createAggregate("my_sum", 1, 0, new SumAggregate());

    SQLException exception =
        assertThrows(
            SQLException.class,
            () ->
                queryFirstColumn(
                    "SELECT my_sum(v) OVER (ORDER BY id ROWS BETWEEN 1 PRECEDING AND CURRENT ROW)"
                        + " FROM readings"));
    assertTrue(
        exception.getMessage().contains("may not be used as a window function"),
        exception.getMessage());
  }

  @Test
  void an_exception_from_an_aggregate_fails_the_statement() throws Exception {
    execute("CREATE TABLE t (v INTEGER)");
    execute("INSERT INTO t VALUES (1)");

    connection.createAggregate(
        "explode",
        1,
        0,
        new Aggregate<Long>() {
          @Override
          public Long init() {
            return 0L;
          }

          @Override
          public Long step(Long state, Object[] args) {
            throw new IllegalStateException("step failed");
          }

          @Override
          public Object result(Long state) {
            return state;
          }
        });

    SQLException exception =
        assertThrows(SQLException.class, () -> queryOneValue("SELECT explode(v) FROM t"));
    assertTrue(
        exception.getMessage().contains("user-defined function raised exception: step failed"),
        exception.getMessage());
  }

  @Test
  void removing_a_function_makes_it_unknown_again() throws Exception {
    connection.createFunction("temporary_fn", 0, 0, args -> 1L);
    assertEquals(1L, queryOneValue("SELECT temporary_fn()"));

    connection.removeFunction("temporary_fn", 0);

    SQLException exception =
        assertThrows(SQLException.class, () -> queryOneValue("SELECT temporary_fn()"));
    assertTrue(exception.getMessage().contains("no such function"), exception.getMessage());
  }

  @Test
  void removing_a_function_that_was_never_registered_does_nothing() throws Exception {
    connection.removeFunction("never_registered", 1);
  }

  @Test
  void registering_the_same_name_and_argument_count_again_replaces_the_function() throws Exception {
    connection.createFunction("pick", 1, 0, args -> "first");
    connection.createFunction("pick", 1, 0, args -> "second");

    assertEquals("second", queryOneValue("SELECT pick(1)"));
  }

  private static class SumAggregate extends Aggregate<Long> {
    @Override
    public Long init() {
      return 0L;
    }

    @Override
    public Long step(Long state, Object[] args) {
      return state + (Long) args[0];
    }

    @Override
    public Object result(Long state) {
      return state;
    }
  }

  private static class SumWindowFunction extends WindowFunction<Long> {
    @Override
    public Long init() {
      return 0L;
    }

    @Override
    public Long step(Long state, Object[] args) {
      return state + (Long) args[0];
    }

    @Override
    public Object result(Long state) {
      return state;
    }

    @Override
    public Object value(Long state) {
      return state;
    }

    @Override
    public Long inverse(Long state, Object[] args) {
      return state - (Long) args[0];
    }
  }

  private void execute(String sql) throws SQLException {
    try (Statement statement = connection.createStatement()) {
      statement.execute(sql);
    }
  }

  private Object queryOneValue(String sql) throws SQLException {
    List<Object> values = queryFirstColumn(sql);
    assertEquals(1, values.size(), "expected exactly one row from: " + sql);
    return values.get(0);
  }

  private List<Object> queryFirstColumn(String sql) throws SQLException {
    try (Statement statement = connection.createStatement()) {
      ResultSet resultSet = statement.executeQuery(sql);
      List<Object> values = new ArrayList<>();
      while (resultSet.next()) {
        values.add(resultSet.getObject(1));
      }
      return values;
    }
  }
}
