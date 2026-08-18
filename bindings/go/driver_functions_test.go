package turso

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"strings"
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
)

// functionConn pins one driver connection: functions are per-connection.
func functionConn(t *testing.T) *sql.Conn {
	t.Helper()
	db := openMem(t)
	db.SetMaxOpenConns(1)
	conn, err := db.Conn(context.Background())
	require.NoError(t, err)
	t.Cleanup(func() { _ = conn.Close() })
	return conn
}

func onFunctions(t *testing.T, conn *sql.Conn, do func(Functions) error) error {
	t.Helper()
	var inner error
	require.NoError(t, conn.Raw(func(dc any) error {
		fns, ok := dc.(Functions)
		require.True(t, ok, "driver connection must expose turso.Functions")
		inner = do(fns)
		return nil
	}))
	return inner
}

// queryError returns the error from preparing the query or from stepping it.
func queryError(conn *sql.Conn, sql string) error {
	rows, err := conn.QueryContext(context.Background(), sql)
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
	}
	return rows.Err()
}

func TestCreateFunctionWithArguments(t *testing.T) {
	conn := functionConn(t)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateFunction("add3", 3, func(args []any) (any, error) {
			var sum int64
			for _, arg := range args {
				n, ok := arg.(int64)
				if !ok {
					return nil, fmt.Errorf("add3 wants integers, got %T", arg)
				}
				sum += n
			}
			return sum, nil
		}, nil)
	}))

	var got int64
	require.NoError(t, conn.QueryRowContext(context.Background(), "SELECT add3(1, 2, 3)").Scan(&got))
	require.Equal(t, int64(6), got)

	err := queryError(conn, "SELECT add3(1, 2)")
	require.Error(t, err)
}

func TestCreateFunctionVariadic(t *testing.T) {
	conn := functionConn(t)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateFunction("joinall", -1, func(args []any) (any, error) {
			parts := make([]string, 0, len(args))
			for _, arg := range args {
				parts = append(parts, fmt.Sprint(arg))
			}
			return strings.Join(parts, ","), nil
		}, nil)
	}))

	for query, want := range map[string]string{
		"SELECT joinall()":             "",
		"SELECT joinall('a')":          "a",
		"SELECT joinall('a', 2)":       "a,2",
		"SELECT joinall('a', 2, 3.5)":  "a,2,3.5",
		"SELECT joinall('a', 2, NULL)": "a,2,<nil>",
		"SELECT joinall(x'0102', 'b')": "[1 2],b",
	} {
		var got string
		require.NoError(t, conn.QueryRowContext(context.Background(), query).Scan(&got), query)
		require.Equal(t, want, got, query)
	}
}

func TestCreateFunctionDeterministicIsFoldedOncePerStatement(t *testing.T) {
	conn := functionConn(t)
	_, err := conn.ExecContext(context.Background(), "CREATE TABLE t(x)")
	require.NoError(t, err)
	_, err = conn.ExecContext(context.Background(), "INSERT INTO t VALUES (1), (2), (3), (4), (5)")
	require.NoError(t, err)

	deterministicCalls := 0
	volatileCalls := 0
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		if err := f.CreateFunction("det_echo", 1, func(args []any) (any, error) {
			deterministicCalls++
			return args[0], nil
		}, &FunctionOptions{Deterministic: true}); err != nil {
			return err
		}
		return f.CreateFunction("vol_echo", 1, func(args []any) (any, error) {
			volatileCalls++
			return args[0], nil
		}, nil)
	}))

	require.NoError(t, queryError(conn, "SELECT det_echo('c') FROM t"))
	require.NoError(t, queryError(conn, "SELECT vol_echo('c') FROM t"))

	// A constant argument to a deterministic function is folded once.
	require.Equal(t, 1, deterministicCalls)
	require.Equal(t, 5, volatileCalls)
}

func TestCreateFunctionReturnTypes(t *testing.T) {
	conn := functionConn(t)
	var next any
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateFunction("echo", 0, func([]any) (any, error) {
			return next, nil
		}, nil)
	}))

	cases := []struct {
		give any
		want any
	}{
		{nil, nil},
		{int64(42), int64(42)},
		{int(7), int64(7)},
		{int32(-3), int64(-3)},
		{uint16(9), int64(9)},
		{true, int64(1)},
		{false, int64(0)},
		{3.5, 3.5},
		{float32(0.5), 0.5},
		{"hello", "hello"},
		{"", ""},
		{[]byte{1, 2, 3}, []byte{1, 2, 3}},
		// The driver reads any empty blob as a nil slice.
		{[]byte{}, []byte(nil)},
	}
	for _, tc := range cases {
		next = tc.give
		var got any
		require.NoError(t, conn.QueryRowContext(context.Background(), "SELECT echo()").Scan(&got), "%v", tc.give)
		require.Equal(t, tc.want, got, "%v", tc.give)
	}

	next = struct{ A int }{1}
	err := queryError(conn, "SELECT echo()")
	require.Error(t, err)
	require.Contains(t, err.Error(), "unsupported type")
}

func TestCreateFunctionErrorSurfaces(t *testing.T) {
	conn := functionConn(t)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateFunction("boom", 0, func([]any) (any, error) {
			return nil, errors.New("kaboom")
		}, nil)
	}))

	err := queryError(conn, "SELECT boom()")
	require.Error(t, err)
	require.Contains(t, err.Error(), "kaboom")

	_, err = conn.ExecContext(context.Background(), "SELECT boom()")
	require.Error(t, err)
	require.Contains(t, err.Error(), "kaboom")
}

func TestCreateFunctionPanicBecomesError(t *testing.T) {
	conn := functionConn(t)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateFunction("panics", 0, func([]any) (any, error) {
			panic("go away")
		}, nil)
	}))

	err := queryError(conn, "SELECT panics()")
	require.Error(t, err)
	require.Contains(t, err.Error(), "go away")
}

// sumAggregate has no Value/Inverse, so it may not be used with OVER.
type sumAggregate struct{ total int64 }

func (a *sumAggregate) Step(args []any) error {
	n, ok := args[0].(int64)
	if !ok {
		return fmt.Errorf("gosum wants integers, got %T", args[0])
	}
	a.total += n
	return nil
}

func (a *sumAggregate) Final() (any, error) { return a.total, nil }

// movingSumAggregate is sumAggregate plus Value/Inverse.
type movingSumAggregate struct{ sumAggregate }

func (a *movingSumAggregate) Value() (any, error) { return a.total, nil }

func (a *movingSumAggregate) Inverse(args []any) error {
	a.total -= args[0].(int64)
	return nil
}

func seedNumbers(t *testing.T, conn *sql.Conn) {
	t.Helper()
	ctx := context.Background()
	_, err := conn.ExecContext(ctx, "CREATE TABLE t (id INTEGER PRIMARY KEY, g INTEGER, n INTEGER)")
	require.NoError(t, err)
	_, err = conn.ExecContext(ctx, "INSERT INTO t (id, g, n) VALUES (1, 1, 10), (2, 1, 20), (3, 2, 30), (4, 2, 40)")
	require.NoError(t, err)
}

func TestCreateAggregateWithGroupBy(t *testing.T) {
	conn := functionConn(t)
	seedNumbers(t, conn)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateAggregate("gosum", 1, func() Aggregate { return &sumAggregate{} }, nil)
	}))

	rows, err := conn.QueryContext(context.Background(), "SELECT g, gosum(n) FROM t GROUP BY g ORDER BY g")
	require.NoError(t, err)
	defer rows.Close()
	got := map[int64]int64{}
	for rows.Next() {
		var g, sum int64
		require.NoError(t, rows.Scan(&g, &sum))
		got[g] = sum
	}
	require.NoError(t, rows.Err())
	require.Equal(t, map[int64]int64{1: 30, 2: 70}, got)

	var empty int64
	require.NoError(t, conn.QueryRowContext(context.Background(), "SELECT gosum(n) FROM t WHERE n > 1000").Scan(&empty))
	require.Equal(t, int64(0), empty)
}

func TestCreateAggregateAsWindowWithMovingFrame(t *testing.T) {
	conn := functionConn(t)
	seedNumbers(t, conn)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateAggregate("gomovingsum", 1, func() Aggregate { return &movingSumAggregate{} }, nil)
	}))

	rows, err := conn.QueryContext(context.Background(),
		"SELECT gomovingsum(n) OVER (ORDER BY id ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) FROM t ORDER BY id")
	require.NoError(t, err)
	defer rows.Close()
	var got []int64
	for rows.Next() {
		var sum int64
		require.NoError(t, rows.Scan(&sum))
		got = append(got, sum)
	}
	require.NoError(t, rows.Err())
	require.Equal(t, []int64{10, 30, 50, 70}, got)
}

func TestCreateAggregateWithoutWindowSupportRejectsOver(t *testing.T) {
	conn := functionConn(t)
	seedNumbers(t, conn)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateAggregate("gosum", 1, func() Aggregate { return &sumAggregate{} }, nil)
	}))

	err := queryError(conn, "SELECT gosum(n) OVER (ORDER BY id) FROM t")
	require.Error(t, err)
	require.Contains(t, err.Error(), "may not be used as a window function")
}

func TestRemoveFunction(t *testing.T) {
	conn := functionConn(t)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		if err := f.CreateFunction("goodbye", 1, func(args []any) (any, error) {
			return args[0], nil
		}, nil); err != nil {
			return err
		}
		// RemoveFunction takes both arities.
		return f.CreateFunction("goodbye", 2, func(args []any) (any, error) {
			return args[0], nil
		}, nil)
	}))

	require.NoError(t, queryError(conn, "SELECT goodbye(1)"))
	require.NoError(t, queryError(conn, "SELECT goodbye(1, 2)"))

	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.RemoveFunction("goodbye")
	}))

	for _, query := range []string{"SELECT goodbye(1)", "SELECT goodbye(1, 2)"} {
		err := queryError(conn, query)
		require.Error(t, err, query)
		require.Contains(t, err.Error(), "no such function", query)
	}
}

func TestManyFunctionsShareTheTrampolines(t *testing.T) {
	conn := functionConn(t)
	const count = 60
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		for i := 0; i < count; i++ {
			offset := int64(i)
			name := fmt.Sprintf("plus%d", i)
			if err := f.CreateFunction(name, 1, func(args []any) (any, error) {
				return args[0].(int64) + offset, nil
			}, nil); err != nil {
				return err
			}
		}
		return nil
	}))

	for i := 0; i < count; i++ {
		var got int64
		query := fmt.Sprintf("SELECT plus%d(100)", i)
		require.NoError(t, conn.QueryRowContext(context.Background(), query).Scan(&got), query)
		require.Equal(t, int64(100+i), got, query)
	}

	functions, _ := udfCountRegistrations()
	require.GreaterOrEqual(t, functions, count)
}

func TestScalarFunctionOverManyRows(t *testing.T) {
	conn := functionConn(t)
	ctx := context.Background()
	_, err := conn.ExecContext(ctx, "CREATE TABLE big (n INTEGER)")
	require.NoError(t, err)
	_, err = conn.ExecContext(ctx, "INSERT INTO big SELECT value FROM generate_series(1, 20000)")
	require.NoError(t, err)

	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateFunction("label", 1, func(args []any) (any, error) {
			return fmt.Sprintf("row-%d", args[0].(int64)), nil
		}, nil)
	}))

	before := udfPendingCount()
	var count int64
	require.NoError(t, conn.QueryRowContext(ctx,
		"SELECT count(*) FROM (SELECT label(n) AS l FROM big) WHERE l LIKE 'row-%'").Scan(&count))
	require.Equal(t, int64(20000), count)

	// Every text result Go allocated was released.
	require.Equal(t, before, udfPendingCount())
}

func TestAggregateInstancesAreReleased(t *testing.T) {
	conn := functionConn(t)
	seedNumbers(t, conn)
	require.NoError(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateAggregate("gosum", 1, func() Aggregate { return &sumAggregate{} }, nil)
	}))

	require.NoError(t, queryError(conn, "SELECT g, gosum(n) FROM t GROUP BY g"))

	_, aggregates := udfCountRegistrations()
	require.Equal(t, 0, aggregates)
}

func TestCreateFunctionRejectsNilCallbacks(t *testing.T) {
	conn := functionConn(t)
	require.ErrorIs(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateFunction("nope", 1, nil, nil)
	}), ErrTursoFunctionInvalid)
	require.ErrorIs(t, onFunctions(t, conn, func(f Functions) error {
		return f.CreateAggregate("nope", 1, nil, nil)
	}), ErrTursoFunctionInvalid)
}

func TestWithFunctionsAppliesToPooledConnections(t *testing.T) {
	connector, err := NewConnector(":memory:", WithFunctions(func(f Functions) error {
		return f.CreateFunction("shout", 1, func(args []any) (any, error) {
			text, ok := args[0].(string)
			if !ok {
				return nil, fmt.Errorf("shout wants text, got %T", args[0])
			}
			return strings.ToUpper(text), nil
		}, &FunctionOptions{Deterministic: true})
	}))
	require.NoError(t, err)

	db := sql.OpenDB(connector)
	t.Cleanup(func() { _ = db.Close() })
	// More than one connection, so a query can land on a later-opened one.
	db.SetMaxOpenConns(3)

	ctx := context.Background()
	const goroutines = 10
	var wg sync.WaitGroup
	errs := make(chan error, goroutines)
	for i := 0; i < goroutines; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			var got string
			if err := db.QueryRowContext(ctx, "SELECT shout('x')").Scan(&got); err != nil {
				errs <- err
				return
			}
			if got != "X" {
				errs <- fmt.Errorf("got %q, want %q", got, "X")
			}
		}()
	}
	wg.Wait()
	close(errs)
	for err := range errs {
		t.Error(err)
	}
}

func TestWithFunctionsErrorFailsConnect(t *testing.T) {
	boom := errors.New("boom")
	connector, err := NewConnector(":memory:", WithFunctions(func(f Functions) error {
		return boom
	}))
	require.NoError(t, err)

	db := sql.OpenDB(connector)
	t.Cleanup(func() { _ = db.Close() })

	err = db.Ping()
	require.Error(t, err)
	require.ErrorIs(t, err, boom)
}

func TestWithFunctionsRunsInOrder(t *testing.T) {
	var order []string
	connector, err := NewConnector(":memory:",
		WithFunctions(func(f Functions) error {
			order = append(order, "first")
			return nil
		}),
		WithFunctions(func(f Functions) error {
			order = append(order, "second")
			return nil
		}),
	)
	require.NoError(t, err)

	db := sql.OpenDB(connector)
	t.Cleanup(func() { _ = db.Close() })
	require.NoError(t, db.Ping())
	require.Equal(t, []string{"first", "second"}, order)
}

func TestPlainOpenIsUnaffectedByConnector(t *testing.T) {
	db := openMem(t)
	require.NoError(t, db.Ping())
}
