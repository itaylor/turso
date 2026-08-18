using System.Runtime.InteropServices;
using Turso.Raw.Public;

namespace Turso.Data.Sqlite;

public partial class SqliteConnection
{
    /// <summary>SQLITE_DETERMINISTIC: the same bit value SQLite uses in <c>sqlite3_create_window_function</c> flags.</summary>
    private const uint DeterministicFlag = 0x800;

    private static readonly TursoAggregateInitCallback WindowInitCallback = InitializeWindowAggregate;
    private static readonly TursoAggregateStepCallback WindowStepCallback = StepWindowAggregate;
    private static readonly TursoAggregateFinalCallback WindowFinalCallback = FinalizeWindowAggregate;
    private static readonly TursoAggregateValueCallback WindowValueCallback = InvokeWindowValue;
    private static readonly TursoAggregateInverseCallback WindowInverseCallback = InvokeWindowInverse;
    private static readonly TursoContextDestructorCallback WindowDestructorCallback = DestroyWindowAggregate;
    private readonly Dictionary<string, WindowFunctionRegistration> _windowFunctions = new(StringComparer.OrdinalIgnoreCase);

    private void RegisterWindowFunction(
        string name,
        int argc,
        bool isDeterministic,
        object? seed,
        Func<object?, object?[], object?>? step,
        Func<object?, object?[], object?>? inverse,
        Func<object?, object?>? value,
        Func<object?, object?> resultSelector)
    {
        ArgumentNullException.ThrowIfNull(name);
        if (step is null)
        {
            _windowFunctions.Remove(name);
            if (_database is not null)
                TursoBindings.UnregisterFunction(DatabaseHandle, name);
            return;
        }

        ArgumentNullException.ThrowIfNull(inverse);
        ArgumentNullException.ThrowIfNull(value);

        var registration = new WindowFunctionRegistration(name, argc, isDeterministic, seed, step, inverse, value, resultSelector);
        _windowFunctions[name] = registration;
        if (_database is not null)
            _nativeFunctionContexts.Add(registration.Register(DatabaseHandle));
    }

    private void RegisterWindowFunctions()
    {
        foreach (var registration in _windowFunctions.Values)
            _nativeFunctionContexts.Add(registration.Register(DatabaseHandle));
    }

    private static IntPtr InitializeWindowAggregate(IntPtr context)
    {
        var registration = (WindowFunctionRegistration?)GCHandle.FromIntPtr(context).Target
            ?? throw new ObjectDisposedException(nameof(WindowFunctionRegistration));
        return registration.CreateInvocationHandle();
    }

    private static TursoExtensionValue StepWindowAggregate(IntPtr context, IntPtr aggregateContext, int argc, IntPtr argv)
    {
        try
        {
            var invocation = (WindowInvocation?)GCHandle.FromIntPtr(aggregateContext).Target
                ?? throw new ObjectDisposedException(nameof(WindowInvocation));
            invocation.Step(ReadArguments(argc, argv));
            return CreateResult(null);
        }
        catch (SqliteException ex)
        {
            return CreateError("__turso_sqlite_error__:" + ex.SqliteErrorCode.ToString(System.Globalization.CultureInfo.InvariantCulture) + ":" + ex.Message);
        }
        catch (Exception ex)
        {
            return CreateError(ex.Message);
        }
    }

    private static TursoExtensionValue FinalizeWindowAggregate(IntPtr context, IntPtr aggregateContext)
    {
        try
        {
            var invocation = (WindowInvocation?)GCHandle.FromIntPtr(aggregateContext).Target
                ?? throw new ObjectDisposedException(nameof(WindowInvocation));
            return CreateResult(invocation.FinalizeResult());
        }
        catch (SqliteException ex)
        {
            return CreateError("__turso_sqlite_error__:" + ex.SqliteErrorCode.ToString(System.Globalization.CultureInfo.InvariantCulture) + ":" + ex.Message);
        }
        catch (Exception ex)
        {
            return CreateError(ex.Message);
        }
    }

    private static TursoExtensionValue InvokeWindowValue(IntPtr context, IntPtr aggregateContext)
    {
        try
        {
            var invocation = (WindowInvocation?)GCHandle.FromIntPtr(aggregateContext).Target
                ?? throw new ObjectDisposedException(nameof(WindowInvocation));
            return CreateResult(invocation.Value());
        }
        catch (SqliteException ex)
        {
            return CreateError("__turso_sqlite_error__:" + ex.SqliteErrorCode.ToString(System.Globalization.CultureInfo.InvariantCulture) + ":" + ex.Message);
        }
        catch (Exception ex)
        {
            return CreateError(ex.Message);
        }
    }

    private static TursoExtensionValue InvokeWindowInverse(IntPtr context, IntPtr aggregateContext, int argc, IntPtr argv)
    {
        try
        {
            var invocation = (WindowInvocation?)GCHandle.FromIntPtr(aggregateContext).Target
                ?? throw new ObjectDisposedException(nameof(WindowInvocation));
            invocation.Inverse(ReadArguments(argc, argv));
            return CreateResult(null);
        }
        catch (SqliteException ex)
        {
            return CreateError("__turso_sqlite_error__:" + ex.SqliteErrorCode.ToString(System.Globalization.CultureInfo.InvariantCulture) + ":" + ex.Message);
        }
        catch (Exception ex)
        {
            return CreateError(ex.Message);
        }
    }

    private static void DestroyWindowAggregate(IntPtr aggregateContext)
    {
        if (aggregateContext == IntPtr.Zero)
            return;

        var handle = GCHandle.FromIntPtr(aggregateContext);
        if (handle.Target is WindowInvocation invocation)
            invocation.Registration.FreeInvocation(handle);
        else if (handle.IsAllocated)
            handle.Free();
    }

    private sealed class WindowFunctionRegistration(
        string name,
        int argc,
        bool isDeterministic,
        object? seed,
        Func<object?, object?[], object?> step,
        Func<object?, object?[], object?> inverse,
        Func<object?, object?> value,
        Func<object?, object?> resultSelector)
    {
        private readonly List<GCHandle> _invocations = [];

        public IntPtr CreateInvocationHandle()
        {
            var handle = GCHandle.Alloc(new WindowInvocation(this, seed, step, inverse, value, resultSelector));
            lock (_invocations)
            {
                _invocations.Add(handle);
            }

            return GCHandle.ToIntPtr(handle);
        }

        public void FreeInvocation(GCHandle handle)
        {
            lock (_invocations)
            {
                _invocations.Remove(handle);
            }

            if (handle.IsAllocated)
                handle.Free();
        }

        public void FreeInvocations()
        {
            lock (_invocations)
            {
                foreach (var handle in _invocations)
                {
                    if (handle.IsAllocated)
                        handle.Free();
                }

                _invocations.Clear();
            }
        }

        public GCHandle Register(Turso.Raw.Public.Handles.TursoDatabaseHandle database)
        {
            var handle = GCHandle.Alloc(this);
            try
            {
                var flags = isDeterministic ? DeterministicFlag : 0u;
                TursoBindings.RegisterWindowFunction(
                    database,
                    name,
                    argc,
                    flags,
                    GCHandle.ToIntPtr(handle),
                    WindowInitCallback,
                    WindowStepCallback,
                    WindowFinalCallback,
                    WindowValueCallback,
                    WindowInverseCallback,
                    ContextDestructorCallback,
                    WindowDestructorCallback,
                    ValueDestructorCallback);
                return handle;
            }
            catch
            {
                handle.Free();
                throw;
            }
        }
    }

    private sealed class WindowInvocation(
        WindowFunctionRegistration registration,
        object? seed,
        Func<object?, object?[], object?> step,
        Func<object?, object?[], object?> inverse,
        Func<object?, object?> value,
        Func<object?, object?> resultSelector)
    {
        private object? _accumulator = seed;

        public WindowFunctionRegistration Registration { get; } = registration;

        public void Step(object?[] args)
        {
            _accumulator = step(_accumulator, args);
        }

        public void Inverse(object?[] args)
        {
            _accumulator = inverse(_accumulator, args);
        }

        public object? Value()
            => value(_accumulator);

        public object? FinalizeResult()
            => resultSelector(_accumulator);
    }
}
