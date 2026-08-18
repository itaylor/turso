package turso

import (
	"fmt"
	"math"
	"runtime"
	"sync"
	"unsafe"

	"github.com/ebitengine/purego"
)

// User-defined functions over sdk-kit's out-parameter C ABI: purego cannot
// build a callback that returns a struct, so Go uses the `*_ptr` entry points.
//
// purego allows only a few thousand callbacks per process, so there is exactly
// ONE trampoline per callback kind. Turso tells registrations apart by the
// `uintptr context` handle, which indexes the registry maps below.
//
// Go never frees Rust memory and Rust never frees Go memory: the Go payload of
// a text/blob/error result stays pinned in udfPendingValues until Turso calls
// our value destructor.

// turso_extension_value_type_t
const (
	tursoExtValueNull    uint32 = 0
	tursoExtValueInteger uint32 = 1
	tursoExtValueFloat   uint32 = 2
	tursoExtValueText    uint32 = 3
	tursoExtValueBlob    uint32 = 4
	tursoExtValueError   uint32 = 5
)

// turso_extension_result_code_t value for an error that carries a message.
const tursoExtResultCustomError uint32 = 14

// Function flags. Same bit values as SQLite's SQLITE_DETERMINISTIC etc.
const (
	tursoFuncDeterministic uint32 = 0x800
	tursoFuncDirectOnly    uint32 = 0x80000
	tursoFuncInnocuous     uint32 = 0x200000
)

// turso_value_t: a 4-byte tag, 4 bytes of padding, then an 8-byte union.
type turso_value_t struct {
	valueType uint32
	_         uint32
	value     uint64
}

// turso_extension_text_t
type turso_extension_text_t struct {
	subtype uint32
	_       uint32
	text    uintptr
	len     uint32
	_       uint32
}

// turso_extension_blob_t
type turso_extension_blob_t struct {
	data uintptr
	size uint64
}

// turso_extension_error_t
type turso_extension_error_t struct {
	code    uint32
	_       uint32
	message uintptr // turso_extension_text_t*
}

var (
	c_turso_connection_register_scalar_function_ptr    func(self TursoConnection, name string, argc int32, flags uint32, context uintptr, callback uintptr, contextDestructor uintptr, valueDestructor uintptr, errorOptOut **byte) turso_status_code_t
	c_turso_connection_register_aggregate_function_ptr func(self TursoConnection, name string, argc int32, flags uint32, context uintptr, init uintptr, step uintptr, finalize uintptr, value uintptr, inverse uintptr, contextDestructor uintptr, aggregateDestructor uintptr, valueDestructor uintptr, errorOptOut **byte) turso_status_code_t
	c_turso_connection_unregister_function             func(self TursoConnection, name string, errorOptOut **byte) turso_status_code_t
)

func registerTursoFunctions(handle uintptr) {
	purego.RegisterLibFunc(&c_turso_connection_register_scalar_function_ptr, handle, "turso_connection_register_scalar_function_ptr")
	purego.RegisterLibFunc(&c_turso_connection_register_aggregate_function_ptr, handle, "turso_connection_register_aggregate_function_ptr")
	purego.RegisterLibFunc(&c_turso_connection_unregister_function, handle, "turso_connection_unregister_function")
}

// ------- registry of registered functions and live aggregates --------

// udfFunction is one registration, keyed by the `context` handle its callbacks
// receive.
type udfFunction struct {
	scalar       ScalarFunc
	newAggregate func() Aggregate
}

// udfAggregate is one per-group accumulator, keyed by what our init callback
// returns as the `turso_agg_ctx_t *`. Turso never dereferences that value, so a
// handle is enough and no C memory is needed.
type udfAggregate struct {
	agg Aggregate
	// Non-nil when agg also implements WindowAggregate.
	window WindowAggregate
}

var (
	udfMu            sync.RWMutex
	udfFunctions     = map[uintptr]*udfFunction{}
	udfAggregates    = map[uintptr]*udfAggregate{}
	udfNextHandle    uintptr
	udfNextAggHandle uintptr
)

func udfRegisterFunction(fn *udfFunction) uintptr {
	udfMu.Lock()
	defer udfMu.Unlock()
	udfNextHandle++
	udfFunctions[udfNextHandle] = fn
	return udfNextHandle
}

func udfLookupFunction(handle uintptr) *udfFunction {
	udfMu.RLock()
	defer udfMu.RUnlock()
	return udfFunctions[handle]
}

func udfForgetFunction(handle uintptr) {
	udfMu.Lock()
	defer udfMu.Unlock()
	delete(udfFunctions, handle)
}

func udfRegisterAggregate(agg *udfAggregate) uintptr {
	udfMu.Lock()
	defer udfMu.Unlock()
	udfNextAggHandle++
	udfAggregates[udfNextAggHandle] = agg
	return udfNextAggHandle
}

func udfLookupAggregate(handle uintptr) *udfAggregate {
	udfMu.RLock()
	defer udfMu.RUnlock()
	return udfAggregates[handle]
}

func udfForgetAggregate(handle uintptr) {
	udfMu.Lock()
	defer udfMu.Unlock()
	delete(udfAggregates, handle)
}

// udfCountRegistrations is only used by tests.
func udfCountRegistrations() (functions int, aggregates int) {
	udfMu.RLock()
	defer udfMu.RUnlock()
	return len(udfFunctions), len(udfAggregates)
}

// ------- results: Go memory Turso reads after the callback returns --------

// udfPendingValue keeps one result's Go memory pinned until Turso has copied it.
type udfPendingValue struct {
	pinner *runtime.Pinner
}

var (
	udfPendingMu     sync.Mutex
	udfPendingValues = map[uintptr]*udfPendingValue{}
)

func udfKeepPinned(key uintptr, pinner *runtime.Pinner) {
	udfPendingMu.Lock()
	udfPendingValues[key] = &udfPendingValue{pinner: pinner}
	udfPendingMu.Unlock()
}

func udfReleasePinned(key uintptr) {
	udfPendingMu.Lock()
	pending := udfPendingValues[key]
	delete(udfPendingValues, key)
	udfPendingMu.Unlock()
	if pending != nil {
		pending.pinner.Unpin()
	}
}

// udfPendingCount is only used by tests, to catch a destructor leak.
func udfPendingCount() int {
	udfPendingMu.Lock()
	defer udfPendingMu.Unlock()
	return len(udfPendingValues)
}

func udfWriteNull(out *turso_value_t) {
	switch out.valueType {
	case tursoExtValueText, tursoExtValueBlob, tursoExtValueError:
		if key := uintptr(out.value); key != 0 {
			udfReleasePinned(key)
		}
	}
	out.valueType = tursoExtValueNull
	out.value = 0
}

func udfWriteInteger(out *turso_value_t, n int64) {
	out.valueType = tursoExtValueInteger
	out.value = uint64(n)
}

func udfWriteFloat(out *turso_value_t, f float64) {
	out.valueType = tursoExtValueFloat
	out.value = math.Float64bits(f)
}

func udfWriteText(out *turso_value_t, s string) {
	buf := []byte(s)
	header := &turso_extension_text_t{len: uint32(len(buf))}
	pinner := &runtime.Pinner{}
	pinner.Pin(header)
	if len(buf) > 0 {
		pinner.Pin(&buf[0])
		header.text = uintptr(unsafe.Pointer(&buf[0]))
	}
	key := uintptr(unsafe.Pointer(header))
	udfKeepPinned(key, pinner)
	out.valueType = tursoExtValueText
	out.value = uint64(key)
}

func udfWriteBlob(out *turso_value_t, b []byte) {
	header := &turso_extension_blob_t{size: uint64(len(b))}
	pinner := &runtime.Pinner{}
	pinner.Pin(header)
	if len(b) > 0 {
		pinner.Pin(&b[0])
		header.data = uintptr(unsafe.Pointer(&b[0]))
	}
	key := uintptr(unsafe.Pointer(header))
	udfKeepPinned(key, pinner)
	out.valueType = tursoExtValueBlob
	out.value = uint64(key)
}

// udfWriteError makes the statement fail: Turso turns an Error result into a
// query error carrying msg.
func udfWriteError(out *turso_value_t, msg string) {
	udfWriteNull(out)
	if msg == "" {
		msg = "turso: user-defined function failed"
	}
	buf := []byte(msg)
	text := &turso_extension_text_t{len: uint32(len(buf))}
	failure := &turso_extension_error_t{code: tursoExtResultCustomError}
	pinner := &runtime.Pinner{}
	pinner.Pin(text)
	pinner.Pin(failure)
	pinner.Pin(&buf[0])
	text.text = uintptr(unsafe.Pointer(&buf[0]))
	failure.message = uintptr(unsafe.Pointer(text))
	key := uintptr(unsafe.Pointer(failure))
	udfKeepPinned(key, pinner)
	out.valueType = tursoExtValueError
	out.value = uint64(key)
}

// Unsupported types are reported as an error rather than silently becoming NULL.
func udfWriteValue(out *turso_value_t, v any) error {
	switch x := v.(type) {
	case nil:
		udfWriteNull(out)
	case bool:
		n := int64(0)
		if x {
			n = 1
		}
		udfWriteInteger(out, n)
	case int:
		udfWriteInteger(out, int64(x))
	case int8:
		udfWriteInteger(out, int64(x))
	case int16:
		udfWriteInteger(out, int64(x))
	case int32:
		udfWriteInteger(out, int64(x))
	case int64:
		udfWriteInteger(out, x)
	case uint:
		udfWriteInteger(out, int64(x))
	case uint8:
		udfWriteInteger(out, int64(x))
	case uint16:
		udfWriteInteger(out, int64(x))
	case uint32:
		udfWriteInteger(out, int64(x))
	case uint64:
		udfWriteInteger(out, int64(x))
	case float32:
		udfWriteFloat(out, float64(x))
	case float64:
		udfWriteFloat(out, x)
	case string:
		udfWriteText(out, x)
	case []byte:
		if x == nil {
			udfWriteNull(out)
		} else {
			udfWriteBlob(out, x)
		}
	default:
		return fmt.Errorf("turso: user-defined function returned unsupported type %T", v)
	}
	return nil
}

// ------- arguments: Turso memory, valid only during the call --------

// Turso's argument memory is only valid during the callback, so every text and
// blob is copied out.
func udfArgs(argc int32, argv uintptr) []any {
	if argc <= 0 || argv == 0 {
		return nil
	}
	args := make([]any, argc)
	stride := unsafe.Sizeof(turso_value_t{})
	for i := int32(0); i < argc; i++ {
		args[i] = udfReadValue((*turso_value_t)(unsafe.Pointer(argv + uintptr(i)*stride)))
	}
	return args
}

func udfReadValue(v *turso_value_t) any {
	switch v.valueType {
	case tursoExtValueInteger:
		return int64(v.value)
	case tursoExtValueFloat:
		return math.Float64frombits(v.value)
	case tursoExtValueText:
		text := (*turso_extension_text_t)(unsafe.Pointer(uintptr(v.value)))
		if text == nil || text.text == 0 || text.len == 0 {
			return ""
		}
		return string(unsafe.Slice((*byte)(unsafe.Pointer(text.text)), int(text.len)))
	case tursoExtValueBlob:
		blob := (*turso_extension_blob_t)(unsafe.Pointer(uintptr(v.value)))
		if blob == nil || blob.data == 0 || blob.size == 0 {
			return []byte{}
		}
		out := make([]byte, int(blob.size))
		copy(out, unsafe.Slice((*byte)(unsafe.Pointer(blob.data)), int(blob.size)))
		return out
	default:
		// NULL, and an Error value which cannot appear in an argument.
		return nil
	}
}

// ------- the nine trampolines --------

var (
	udfScalarCallback              uintptr
	udfAggregateInitCallback       uintptr
	udfAggregateStepCallback       uintptr
	udfAggregateFinalCallback      uintptr
	udfAggregateValueCallback      uintptr
	udfAggregateInverseCallback    uintptr
	udfContextDestructorCallback   uintptr
	udfAggregateDestructorCallback uintptr
	udfValueDestructorCallback     uintptr
)

func init() {
	udfScalarCallback = purego.NewCallback(udfScalarTrampoline)
	udfAggregateInitCallback = purego.NewCallback(udfAggregateInitTrampoline)
	udfAggregateStepCallback = purego.NewCallback(udfAggregateStepTrampoline)
	udfAggregateFinalCallback = purego.NewCallback(udfAggregateFinalTrampoline)
	udfAggregateValueCallback = purego.NewCallback(udfAggregateValueTrampoline)
	udfAggregateInverseCallback = purego.NewCallback(udfAggregateInverseTrampoline)
	udfContextDestructorCallback = purego.NewCallback(udfContextDestructorTrampoline)
	udfAggregateDestructorCallback = purego.NewCallback(udfAggregateDestructorTrampoline)
	udfValueDestructorCallback = purego.NewCallback(udfValueDestructorTrampoline)
}

// A panic must not unwind into Rust, which would abort the process.
func udfRecover(out *turso_value_t) {
	if r := recover(); r != nil {
		udfWriteError(out, fmt.Sprintf("turso: user-defined function panicked: %v", r))
	}
}

func udfScalarTrampoline(context uintptr, argc int32, argv uintptr, result uintptr) {
	out := (*turso_value_t)(unsafe.Pointer(result))
	defer udfRecover(out)
	fn := udfLookupFunction(context)
	if fn == nil || fn.scalar == nil {
		udfWriteError(out, "turso: scalar function is no longer registered")
		return
	}
	value, err := fn.scalar(udfArgs(argc, argv))
	if err != nil {
		udfWriteError(out, err.Error())
		return
	}
	if err := udfWriteValue(out, value); err != nil {
		udfWriteError(out, err.Error())
	}
}

func udfAggregateInitTrampoline(context uintptr) (handle uintptr) {
	defer func() {
		if r := recover(); r != nil {
			// A null accumulator has no handle, so the first step reports it.
			handle = 0
		}
	}()
	fn := udfLookupFunction(context)
	if fn == nil || fn.newAggregate == nil {
		return 0
	}
	agg := fn.newAggregate()
	if agg == nil {
		return 0
	}
	window, _ := agg.(WindowAggregate)
	return udfRegisterAggregate(&udfAggregate{agg: agg, window: window})
}

func udfAggregateStepTrampoline(_ uintptr, aggregate uintptr, argc int32, argv uintptr, result uintptr) {
	out := (*turso_value_t)(unsafe.Pointer(result))
	defer udfRecover(out)
	state := udfLookupAggregate(aggregate)
	if state == nil {
		udfWriteError(out, "turso: aggregate could not be created")
		return
	}
	if err := state.agg.Step(udfArgs(argc, argv)); err != nil {
		udfWriteError(out, err.Error())
	}
}

func udfAggregateFinalTrampoline(_ uintptr, aggregate uintptr, result uintptr) {
	out := (*turso_value_t)(unsafe.Pointer(result))
	defer udfRecover(out)
	state := udfLookupAggregate(aggregate)
	if state == nil {
		udfWriteError(out, "turso: aggregate could not be created")
		return
	}
	value, err := state.agg.Final()
	if err != nil {
		udfWriteError(out, err.Error())
		return
	}
	if err := udfWriteValue(out, value); err != nil {
		udfWriteError(out, err.Error())
	}
}

func udfAggregateValueTrampoline(_ uintptr, aggregate uintptr, result uintptr) {
	out := (*turso_value_t)(unsafe.Pointer(result))
	defer udfRecover(out)
	state := udfLookupAggregate(aggregate)
	if state == nil || state.window == nil {
		udfWriteError(out, "turso: aggregate cannot report a running value")
		return
	}
	value, err := state.window.Value()
	if err != nil {
		udfWriteError(out, err.Error())
		return
	}
	if err := udfWriteValue(out, value); err != nil {
		udfWriteError(out, err.Error())
	}
}

func udfAggregateInverseTrampoline(_ uintptr, aggregate uintptr, argc int32, argv uintptr, result uintptr) {
	out := (*turso_value_t)(unsafe.Pointer(result))
	defer udfRecover(out)
	state := udfLookupAggregate(aggregate)
	if state == nil || state.window == nil {
		udfWriteError(out, "turso: aggregate cannot undo a step")
		return
	}
	if err := state.window.Inverse(udfArgs(argc, argv)); err != nil {
		udfWriteError(out, err.Error())
	}
}

// Runs when Turso drops a registration: on replace, on RemoveFunction, on
// connection close, or when the registration itself failed.
func udfContextDestructorTrampoline(context uintptr) {
	udfForgetFunction(context)
}

// Runs after Final, or if the group was abandoned.
func udfAggregateDestructorTrampoline(aggregate uintptr) {
	udfForgetAggregate(aggregate)
}

// Runs once per result value, after Turso has copied it.
func udfValueDestructorTrampoline(result uintptr) {
	if result == 0 {
		return
	}
	udfWriteNull((*turso_value_t)(unsafe.Pointer(result)))
}

// ------- Go wrappers over the register entry points --------

func turso_connection_register_scalar_function_ptr(self TursoConnection, name string, argc int32, flags uint32, context uintptr) error {
	var errPtr *byte
	status := c_turso_connection_register_scalar_function_ptr(
		self, name, argc, flags, context,
		udfScalarCallback,
		udfContextDestructorCallback,
		udfValueDestructorCallback,
		&errPtr,
	)
	if status == int32(TURSO_OK) {
		return nil
	}
	return statusToError(TursoStatusCode(status), decodeAndFreeCString(errPtr))
}

func turso_connection_register_aggregate_function_ptr(self TursoConnection, name string, argc int32, flags uint32, context uintptr, window bool) error {
	value, inverse := uintptr(0), uintptr(0)
	if window {
		value, inverse = udfAggregateValueCallback, udfAggregateInverseCallback
	}
	var errPtr *byte
	status := c_turso_connection_register_aggregate_function_ptr(
		self, name, argc, flags, context,
		udfAggregateInitCallback,
		udfAggregateStepCallback,
		udfAggregateFinalCallback,
		value,
		inverse,
		udfContextDestructorCallback,
		udfAggregateDestructorCallback,
		udfValueDestructorCallback,
		&errPtr,
	)
	if status == int32(TURSO_OK) {
		return nil
	}
	return statusToError(TursoStatusCode(status), decodeAndFreeCString(errPtr))
}

func turso_connection_unregister_function(self TursoConnection, name string) error {
	var errPtr *byte
	status := c_turso_connection_unregister_function(self, name, &errPtr)
	if status == int32(TURSO_OK) {
		return nil
	}
	return statusToError(TursoStatusCode(status), decodeAndFreeCString(errPtr))
}
