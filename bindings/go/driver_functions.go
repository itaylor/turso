package turso

import (
	"errors"
	"fmt"
)

// ScalarFunc is a user-defined scalar SQL function. Arguments arrive as nil
// (NULL), int64, float64, string or []byte; a returned error fails the
// running statement with that message.
type ScalarFunc func(args []any) (any, error)

// Aggregate accumulates one group of a GROUP BY (or one window frame). A fresh
// instance is created for every group.
type Aggregate interface {
	Step(args []any) error
	// Final is called exactly once per group, even when the group had no rows.
	Final() (any, error)
}

// WindowAggregate is an Aggregate usable with OVER (... ROWS BETWEEN ...).
// Using a plain Aggregate with OVER fails the query at run time with
// "may not be used as a window function".
type WindowAggregate interface {
	Aggregate
	// Value reports the accumulator's current value without consuming it.
	Value() (any, error)
	// Inverse undoes an earlier Step for a row that left the frame.
	Inverse(args []any) error
}

// FunctionOptions carries the SQLite function flags. A nil *FunctionOptions
// means no flags.
type FunctionOptions struct {
	// Deterministic promises the same arguments always produce the same result.
	Deterministic bool
	// DirectOnly forbids the function in schema SQL: triggers, views, CHECK
	// constraints, generated columns, index expressions, DEFAULT.
	DirectOnly bool
	// Innocuous promises no I/O and no side effects.
	Innocuous bool
}

func (o *FunctionOptions) flags() uint32 {
	if o == nil {
		return 0
	}
	var flags uint32
	if o.Deterministic {
		flags |= tursoFuncDeterministic
	}
	if o.DirectOnly {
		flags |= tursoFuncDirectOnly
	}
	if o.Innocuous {
		flags |= tursoFuncInnocuous
	}
	return flags
}

// ErrTursoFunctionInvalid reports an unusable registration, such as a nil
// callback.
var ErrTursoFunctionInvalid = errors.New("turso: invalid function registration")

// Functions is the user-defined-function API of a Turso driver connection.
// The driver's connection type is unexported, so reach it through
// (*sql.Conn).Raw:
//
//	conn, err := db.Conn(ctx)
//	err = conn.Raw(func(dc any) error {
//		return dc.(turso.Functions).CreateFunction("triple", 1, triple, nil)
//	})
//
// Functions are per-connection, as in SQLite, so register on a pinned
// *sql.Conn and query on that same connection. See [WithFunctions] to cover
// every connection in a pool instead.
type Functions interface {
	CreateFunction(name string, nArgs int, fn ScalarFunc, opts *FunctionOptions) error
	CreateAggregate(name string, nArgs int, newAggregate func() Aggregate, opts *FunctionOptions) error
	RemoveFunction(name string) error
}

var _ Functions = (*tursoDbConnection)(nil)

// CreateFunction replaces any function already registered under the same name
// and argument count. Pass nArgs = -1 for a variadic function.
func (c *tursoDbConnection) CreateFunction(name string, nArgs int, fn ScalarFunc, opts *FunctionOptions) error {
	if fn == nil {
		return fmt.Errorf("%w: %q has no function", ErrTursoFunctionInvalid, name)
	}
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed || c.conn == nil {
		return ErrTursoConnClosed
	}
	handle := udfRegisterFunction(&udfFunction{scalar: fn})
	if err := turso_connection_register_scalar_function_ptr(c.conn, name, int32(nArgs), opts.flags(), handle); err != nil {
		udfForgetFunction(handle)
		return err
	}
	return nil
}

// CreateAggregate replaces any function already registered under the same name
// and argument count. Pass nArgs = -1 for a variadic aggregate.
//
// newAggregate is called once per group. Window support is decided once, at
// registration, by checking whether one probe instance implements
// [WindowAggregate].
func (c *tursoDbConnection) CreateAggregate(name string, nArgs int, newAggregate func() Aggregate, opts *FunctionOptions) error {
	if newAggregate == nil {
		return fmt.Errorf("%w: %q has no constructor", ErrTursoFunctionInvalid, name)
	}
	probe := newAggregate()
	if probe == nil {
		return fmt.Errorf("%w: %q constructor returned nil", ErrTursoFunctionInvalid, name)
	}
	_, window := probe.(WindowAggregate)

	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed || c.conn == nil {
		return ErrTursoConnClosed
	}
	handle := udfRegisterFunction(&udfFunction{newAggregate: newAggregate})
	if err := turso_connection_register_aggregate_function_ptr(c.conn, name, int32(nArgs), opts.flags(), handle, window); err != nil {
		udfForgetFunction(handle)
		return err
	}
	return nil
}

// RemoveFunction unregisters *every* arity registered under name: the C ABI
// keys removal by name alone.
func (c *tursoDbConnection) RemoveFunction(name string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed || c.conn == nil {
		return ErrTursoConnClosed
	}
	return turso_connection_unregister_function(c.conn, name)
}
