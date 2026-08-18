#include "TursoUdf.h"

#include <cstdlib>
#include <cstring>
#include <mutex>
#include <thread>
#include <unordered_map>
#include <utility>
#include <vector>

extern "C" {
#include <turso.h>
}

namespace turso {

using namespace facebook;

namespace {

// The engine only knows a registration by the uintptr_t context it was made
// with, so the callbacks look it up in g_registrations on every call.
struct Registration {
    jsi::Runtime *runtime = nullptr;
    /// As it appears in error messages, e.g. "user-defined function foo()".
    std::string name;
    std::shared_ptr<jsi::Function> func;
    std::shared_ptr<jsi::Function> start;
    std::shared_ptr<jsi::Function> step;
    std::shared_ptr<jsi::Function> inverse;
    std::shared_ptr<jsi::Function> result;
};

/** One group's (or one window frame's) accumulator. */
struct AggregateState {
    std::shared_ptr<jsi::Value> total;
    /// Set once a callback failed; later callbacks report it unchanged.
    std::string error;
};

std::mutex g_mutex;
std::unordered_map<uintptr_t, std::shared_ptr<Registration>> g_registrations;
uintptr_t g_nextContext = 1;
/// The thread that runs JavaScript, learned from the first registration.
std::thread::id g_jsThread;
bool g_jsThreadKnown = false;
/// True once the runtime is going away and no callback may touch it.
bool g_shuttingDown = false;
std::shared_ptr<react::CallInvoker> g_invoker;

// JSI values may only be touched on the JavaScript thread, but the engine drops
// a registration from whichever thread finalized the statement.
template <typename T>
void releaseOnJsThread(std::shared_ptr<T> owned)
{
    if (!owned)
    {
        return;
    }

    std::shared_ptr<react::CallInvoker> invoker;
    {
        std::lock_guard<std::mutex> lock(g_mutex);
        if (g_jsThreadKnown && std::this_thread::get_id() == g_jsThread)
        {
            return; // Already on the JS thread: `owned` dies here.
        }
        invoker = g_invoker;
    }

    if (!invoker)
    {
        // No way back to the JS thread; destroying JSI values here would
        // corrupt the runtime, so leak them with the runtime's heap.
        auto *leaked = new std::shared_ptr<T>(std::move(owned));
        (void)leaked;
        return;
    }

    invoker->invokeAsync([owned = std::move(owned)](jsi::Runtime &) mutable { owned.reset(); });
}

std::shared_ptr<Registration> lookupRegistration(uintptr_t context)
{
    std::lock_guard<std::mutex> lock(g_mutex);
    if (g_shuttingDown)
    {
        return nullptr;
    }
    auto it = g_registrations.find(context);
    if (it == g_registrations.end())
    {
        return nullptr;
    }
    return it->second;
}

bool onJsThread()
{
    std::lock_guard<std::mutex> lock(g_mutex);
    return g_jsThreadKnown && !g_shuttingDown && std::this_thread::get_id() == g_jsThread;
}

// Called from a host function, so this is where the JS thread is learned.
uintptr_t insertRegistration(const std::shared_ptr<Registration> &registration)
{
    std::lock_guard<std::mutex> lock(g_mutex);
    g_jsThread = std::this_thread::get_id();
    g_jsThreadKnown = true;
    g_shuttingDown = false;
    uintptr_t context = g_nextContext++;
    g_registrations[context] = registration;
    return context;
}

void eraseRegistration(uintptr_t context)
{
    std::shared_ptr<Registration> registration;
    {
        std::lock_guard<std::mutex> lock(g_mutex);
        auto it = g_registrations.find(context);
        if (it == g_registrations.end())
        {
            return;
        }
        registration = std::move(it->second);
        g_registrations.erase(it);
    }
    releaseOnJsThread(std::move(registration));
}

// Every heap block below is malloc'd here and freed in tursoUdfDestroyValue,
// the registered value destructor. Without it the engine would free the memory
// with Rust's allocator.

turso_value_t nullValue()
{
    turso_value_t value{};
    value.value_type = TURSO_EXTENSION_VALUE_NULL;
    return value;
}

// Static so it can be reported without allocating; tursoUdfDestroyValue leaves it alone.
turso_extension_error_t kOutOfMemory{TURSO_EXTENSION_RESULT_OOM, nullptr};

turso_value_t outOfMemoryValue()
{
    turso_value_t value{};
    value.value_type = TURSO_EXTENSION_VALUE_ERROR;
    value.value.error = &kOutOfMemory;
    return value;
}

turso_value_t integerValue(int64_t number)
{
    turso_value_t value{};
    value.value_type = TURSO_EXTENSION_VALUE_INTEGER;
    value.value.int_value = number;
    return value;
}

turso_value_t realValue(double number)
{
    turso_value_t value{};
    value.value_type = TURSO_EXTENSION_VALUE_FLOAT;
    value.value.float_value = number;
    return value;
}

turso_extension_text_t *allocText(const char *text, size_t len)
{
    auto *allocated = static_cast<turso_extension_text_t *>(malloc(sizeof(turso_extension_text_t)));
    if (!allocated)
    {
        return nullptr;
    }
    allocated->subtype = TURSO_EXTENSION_TEXT_TEXT;
    allocated->text = nullptr;
    allocated->len = 0;

    if (len > 0)
    {
        auto *bytes = static_cast<uint8_t *>(malloc(len));
        if (!bytes)
        {
            free(allocated);
            return nullptr;
        }
        memcpy(bytes, text, len);
        allocated->text = bytes;
        allocated->len = static_cast<uint32_t>(len);
    }
    return allocated;
}

void freeText(turso_extension_text_t *text)
{
    if (!text)
    {
        return;
    }
    free(const_cast<uint8_t *>(text->text));
    free(text);
}

turso_value_t textValue(const std::string &text)
{
    turso_extension_text_t *allocated = allocText(text.data(), text.size());
    if (!allocated)
    {
        return outOfMemoryValue();
    }
    turso_value_t value{};
    value.value_type = TURSO_EXTENSION_VALUE_TEXT;
    value.value.text = allocated;
    return value;
}

turso_value_t blobValue(const uint8_t *data, size_t len)
{
    auto *blob = static_cast<turso_extension_blob_t *>(malloc(sizeof(turso_extension_blob_t)));
    if (!blob)
    {
        return outOfMemoryValue();
    }
    blob->data = nullptr;
    blob->size = 0;

    if (len > 0)
    {
        auto *bytes = static_cast<uint8_t *>(malloc(len));
        if (!bytes)
        {
            free(blob);
            return outOfMemoryValue();
        }
        memcpy(bytes, data, len);
        blob->data = bytes;
        blob->size = static_cast<uint64_t>(len);
    }

    turso_value_t value{};
    value.value_type = TURSO_EXTENSION_VALUE_BLOB;
    value.value.blob = blob;
    return value;
}

turso_value_t errorValue(const std::string &message)
{
    auto *error = static_cast<turso_extension_error_t *>(malloc(sizeof(turso_extension_error_t)));
    if (!error)
    {
        return outOfMemoryValue();
    }
    error->code = TURSO_EXTENSION_RESULT_CUSTOM_ERROR;
    error->message = allocText(message.data(), message.size());

    turso_value_t value{};
    value.value_type = TURSO_EXTENSION_VALUE_ERROR;
    value.value.error = error;
    return value;
}

jsi::Value makeArrayBuffer(jsi::Runtime &rt, const uint8_t *data, size_t len)
{
    jsi::Function arrayBufferCtor = rt.global().getPropertyAsFunction(rt, "ArrayBuffer");
    jsi::Object arrayBuffer = arrayBufferCtor.callAsConstructor(rt, static_cast<int>(len)).asObject(rt);
    if (data && len > 0)
    {
        jsi::ArrayBuffer buf = arrayBuffer.getArrayBuffer(rt);
        memcpy(buf.data(rt), data, len);
    }
    return arrayBuffer;
}

jsi::Value valueToJs(jsi::Runtime &rt, const turso_value_t &value)
{
    switch (value.value_type)
    {
        case TURSO_EXTENSION_VALUE_INTEGER:
            // Integers become JavaScript numbers, as column values do.
            return jsi::Value(static_cast<double>(value.value.int_value));

        case TURSO_EXTENSION_VALUE_FLOAT:
            return jsi::Value(value.value.float_value);

        case TURSO_EXTENSION_VALUE_TEXT:
        {
            const turso_extension_text_t *text = value.value.text;
            if (!text || !text->text || text->len == 0)
            {
                return jsi::String::createFromUtf8(rt, "");
            }
            return jsi::String::createFromUtf8(rt, text->text, static_cast<size_t>(text->len));
        }

        case TURSO_EXTENSION_VALUE_BLOB:
        {
            const turso_extension_blob_t *blob = value.value.blob;
            if (!blob)
            {
                return makeArrayBuffer(rt, nullptr, 0);
            }
            return makeArrayBuffer(rt, blob->data, static_cast<size_t>(blob->size));
        }

        case TURSO_EXTENSION_VALUE_NULL:
        case TURSO_EXTENSION_VALUE_ERROR:
        default:
            return jsi::Value::null();
    }
}

// `undefined` and `null` both mean SQL NULL; anything SQL has no type for fails
// the statement, as in better-sqlite3.
turso_value_t jsToValue(jsi::Runtime &rt, const jsi::Value &value, const std::string &what)
{
    if (value.isUndefined() || value.isNull())
    {
        return nullValue();
    }
    if (value.isBool())
    {
        return integerValue(value.getBool() ? 1 : 0);
    }
    if (value.isNumber())
    {
        double number = value.asNumber();
        if (number == static_cast<double>(static_cast<int64_t>(number)) &&
            number >= -9223372036854775808.0 && number < 9223372036854775808.0)
        {
            return integerValue(static_cast<int64_t>(number));
        }
        return realValue(number);
    }
    if (value.isString())
    {
        return textValue(value.asString(rt).utf8(rt));
    }
    if (value.isObject())
    {
        jsi::Object object = value.asObject(rt);
        if (object.isArrayBuffer(rt))
        {
            jsi::ArrayBuffer buffer = object.getArrayBuffer(rt);
            return blobValue(buffer.data(rt), buffer.size(rt));
        }
        // A typed array (Uint8Array and friends) is a view on an ArrayBuffer.
        if (object.hasProperty(rt, "buffer") && object.hasProperty(rt, "byteOffset") &&
            object.hasProperty(rt, "byteLength"))
        {
            jsi::Value bufferValue = object.getProperty(rt, "buffer");
            if (bufferValue.isObject() && bufferValue.asObject(rt).isArrayBuffer(rt))
            {
                jsi::ArrayBuffer buffer = bufferValue.asObject(rt).getArrayBuffer(rt);
                size_t offset = static_cast<size_t>(object.getProperty(rt, "byteOffset").asNumber());
                size_t length = static_cast<size_t>(object.getProperty(rt, "byteLength").asNumber());
                if (offset + length <= buffer.size(rt))
                {
                    return blobValue(buffer.data(rt) + offset, length);
                }
            }
        }
    }
    return errorValue(what + " returned an invalid value");
}

std::string describeThrow(const std::string &what)
{
    return what + " threw";
}

extern "C" {

void tursoUdfDestroyValue(turso_value_t *result)
{
    if (!result)
    {
        return;
    }
    switch (result->value_type)
    {
        case TURSO_EXTENSION_VALUE_TEXT:
            freeText(const_cast<turso_extension_text_t *>(result->value.text));
            break;

        case TURSO_EXTENSION_VALUE_BLOB:
        {
            auto *blob = const_cast<turso_extension_blob_t *>(result->value.blob);
            if (blob)
            {
                free(const_cast<uint8_t *>(blob->data));
                free(blob);
            }
            break;
        }

        case TURSO_EXTENSION_VALUE_ERROR:
        {
            auto *error = const_cast<turso_extension_error_t *>(result->value.error);
            if (error && error != &kOutOfMemory)
            {
                freeText(error->message);
                free(error);
            }
            break;
        }

        default:
            break;
    }
}

void tursoUdfDestroyContext(uintptr_t context)
{
    eraseRegistration(context);
}

void tursoUdfDestroyAggregate(uintptr_t context)
{
    auto *ctx = reinterpret_cast<turso_agg_ctx_t *>(context);
    if (!ctx)
    {
        return;
    }
    auto *state = static_cast<AggregateState *>(ctx->state);
    free(ctx);
    if (!state)
    {
        return;
    }
    std::shared_ptr<jsi::Value> total = std::move(state->total);
    delete state;
    releaseOnJsThread(std::move(total));
}

// The result is written through `out`, which the engine pre-set to NULL.
void tursoUdfCallScalar(
    uintptr_t context,
    int32_t argc,
    const turso_value_t *argv,
    turso_value_t *out)
{
    if (!out)
    {
        return;
    }

    std::shared_ptr<Registration> registration = lookupRegistration(context);
    if (!registration || !registration->func || !registration->runtime)
    {
        *out = errorValue("user-defined function is no longer registered");
        return;
    }
    if (!onJsThread())
    {
        *out = errorValue(
            registration->name + " can only run on the thread that registered it");
        return;
    }

    jsi::Runtime &rt = *registration->runtime;
    try
    {
        std::vector<jsi::Value> args;
        args.reserve(argc > 0 ? static_cast<size_t>(argc) : 0);
        for (int32_t i = 0; i < argc; ++i)
        {
            args.push_back(valueToJs(rt, argv[i]));
        }

        const jsi::Value *argsPtr = args.data();
        jsi::Value result = registration->func->call(rt, argsPtr, args.size());
        *out = jsToValue(rt, result, registration->name);
    }
    catch (const jsi::JSError &err)
    {
        *out = errorValue(registration->name + ": " + err.getMessage());
    }
    catch (const std::exception &err)
    {
        *out = errorValue(registration->name + ": " + err.what());
    }
    catch (...)
    {
        *out = errorValue(describeThrow(registration->name));
    }
}

turso_agg_ctx_t *tursoUdfAggregateInit(uintptr_t context)
{
    auto *ctx = static_cast<turso_agg_ctx_t *>(malloc(sizeof(turso_agg_ctx_t)));
    if (!ctx)
    {
        return nullptr;
    }
    auto *state = new AggregateState();
    ctx->state = state;

    std::shared_ptr<Registration> registration = lookupRegistration(context);
    if (!registration || !registration->start || !registration->runtime)
    {
        state->error = "user-defined aggregate is no longer registered";
        return ctx;
    }
    if (!onJsThread())
    {
        state->error = registration->name + " can only run on the thread that registered it";
        return ctx;
    }

    jsi::Runtime &rt = *registration->runtime;
    try
    {
        // The argument types pick jsi's (const Value*, size_t) overload rather
        // than its variadic one.
        std::vector<jsi::Value> noArgs;
        const jsi::Value *argsPtr = noArgs.data();
        jsi::Value total = registration->start->call(rt, argsPtr, noArgs.size());
        state->total = std::make_shared<jsi::Value>(std::move(total));
    }
    catch (const jsi::JSError &err)
    {
        state->error = registration->name + ": " + err.getMessage();
    }
    catch (const std::exception &err)
    {
        state->error = registration->name + ": " + err.what();
    }
    catch (...)
    {
        state->error = describeThrow(registration->name);
    }
    return ctx;
}

// `step` and `inverse` both take the running total ahead of the SQL arguments
// and return the new total, or `undefined` to keep the old one.
void tursoUdfAccumulate(
    uintptr_t context,
    turso_agg_ctx_t *aggregateContext,
    int32_t argc,
    const turso_value_t *argv,
    turso_value_t *out,
    bool isInverse)
{
    if (!out)
    {
        return;
    }
    if (!aggregateContext || !aggregateContext->state)
    {
        *out = errorValue("user-defined aggregate has no accumulator");
        return;
    }
    auto *state = static_cast<AggregateState *>(aggregateContext->state);
    if (!state->error.empty())
    {
        *out = errorValue(state->error);
        return;
    }

    std::shared_ptr<Registration> registration = lookupRegistration(context);
    if (!registration || !registration->runtime)
    {
        *out = errorValue("user-defined aggregate is no longer registered");
        return;
    }
    const std::shared_ptr<jsi::Function> &callback =
        isInverse ? registration->inverse : registration->step;
    if (!callback)
    {
        *out = errorValue(registration->name + " may not be used as a window function");
        return;
    }
    if (!onJsThread())
    {
        *out = errorValue(
            registration->name + " can only run on the thread that registered it");
        return;
    }

    jsi::Runtime &rt = *registration->runtime;
    try
    {
        std::vector<jsi::Value> args;
        args.reserve((argc > 0 ? static_cast<size_t>(argc) : 0) + 1);
        args.push_back(state->total ? jsi::Value(rt, *state->total) : jsi::Value::undefined());
        for (int32_t i = 0; i < argc; ++i)
        {
            args.push_back(valueToJs(rt, argv[i]));
        }

        const jsi::Value *argsPtr = args.data();
        jsi::Value updated = callback->call(rt, argsPtr, args.size());
        if (!updated.isUndefined())
        {
            state->total = std::make_shared<jsi::Value>(std::move(updated));
        }
        // Success leaves the NULL value the engine pre-set.
        return;
    }
    catch (const jsi::JSError &err)
    {
        state->error = registration->name + ": " + err.getMessage();
    }
    catch (const std::exception &err)
    {
        state->error = registration->name + ": " + err.what();
    }
    catch (...)
    {
        state->error = describeThrow(registration->name);
    }
    *out = errorValue(state->error);
}

void tursoUdfAggregateStep(
    uintptr_t context,
    turso_agg_ctx_t *aggregateContext,
    int32_t argc,
    const turso_value_t *argv,
    turso_value_t *out)
{
    tursoUdfAccumulate(context, aggregateContext, argc, argv, out, false);
}

void tursoUdfAggregateInverse(
    uintptr_t context,
    turso_agg_ctx_t *aggregateContext,
    int32_t argc,
    const turso_value_t *argv,
    turso_value_t *out)
{
    tursoUdfAccumulate(context, aggregateContext, argc, argv, out, true);
}

void tursoUdfAggregateResult(
    uintptr_t context,
    turso_agg_ctx_t *aggregateContext,
    turso_value_t *out)
{
    if (!out)
    {
        return;
    }
    if (!aggregateContext || !aggregateContext->state)
    {
        *out = errorValue("user-defined aggregate has no accumulator");
        return;
    }
    auto *state = static_cast<AggregateState *>(aggregateContext->state);
    if (!state->error.empty())
    {
        *out = errorValue(state->error);
        return;
    }

    std::shared_ptr<Registration> registration = lookupRegistration(context);
    if (!registration || !registration->runtime)
    {
        *out = errorValue("user-defined aggregate is no longer registered");
        return;
    }
    if (!onJsThread())
    {
        *out = errorValue(
            registration->name + " can only run on the thread that registered it");
        return;
    }

    jsi::Runtime &rt = *registration->runtime;
    try
    {
        jsi::Value total = state->total ? jsi::Value(rt, *state->total) : jsi::Value::undefined();
        if (!registration->result)
        {
            *out = jsToValue(rt, total, registration->name);
            return;
        }
        const jsi::Value *argsPtr = &total;
        size_t argCount = 1;
        jsi::Value finalValue = registration->result->call(rt, argsPtr, argCount);
        *out = jsToValue(rt, finalValue, registration->name);
        return;
    }
    catch (const jsi::JSError &err)
    {
        state->error = registration->name + ": " + err.getMessage();
    }
    catch (const std::exception &err)
    {
        state->error = registration->name + ": " + err.what();
    }
    catch (...)
    {
        state->error = describeThrow(registration->name);
    }
    *out = errorValue(state->error);
}

} // extern "C"

} // namespace

void udfSetCallInvoker(const std::shared_ptr<react::CallInvoker> &invoker)
{
    std::lock_guard<std::mutex> lock(g_mutex);
    g_invoker = invoker;
    g_shuttingDown = false;
}

void udfInvalidate()
{
    std::lock_guard<std::mutex> lock(g_mutex);
    g_shuttingDown = true;
    // This may not be the JS thread, so the callbacks are leaked rather than
    // destroyed; their memory belongs to the runtime's heap.
    for (auto &entry : g_registrations)
    {
        auto *leaked = new std::shared_ptr<Registration>(entry.second);
        (void)leaked;
    }
    g_registrations.clear();
    g_jsThreadKnown = false;
    g_invoker.reset();
}

void udfRegisterScalar(
    jsi::Runtime &rt,
    turso_connection_t *conn,
    const std::string &name,
    int32_t argc,
    uint32_t flags,
    std::shared_ptr<jsi::Function> fn)
{
    auto registration = std::make_shared<Registration>();
    registration->runtime = &rt;
    registration->name = "user-defined function " + name + "()";
    registration->func = std::move(fn);

    uintptr_t context = insertRegistration(registration);
    const char *error = nullptr;
    turso_status_code_t status = turso_connection_register_scalar_function_ptr(
        conn,
        name.c_str(),
        argc,
        flags,
        context,
        tursoUdfCallScalar,
        tursoUdfDestroyContext,
        tursoUdfDestroyValue,
        &error);

    if (status != TURSO_OK)
    {
        eraseRegistration(context);
        throw jsi::JSError(rt, error ? error : "failed to register the function");
    }
}

void udfRegisterAggregate(
    jsi::Runtime &rt,
    turso_connection_t *conn,
    const std::string &name,
    int32_t argc,
    uint32_t flags,
    std::shared_ptr<jsi::Function> start,
    std::shared_ptr<jsi::Function> step,
    std::shared_ptr<jsi::Function> inverse,
    std::shared_ptr<jsi::Function> result)
{
    bool isWindow = static_cast<bool>(inverse);

    auto registration = std::make_shared<Registration>();
    registration->runtime = &rt;
    registration->name = "user-defined aggregate " + name + "()";
    registration->start = std::move(start);
    registration->step = std::move(step);
    registration->inverse = std::move(inverse);
    registration->result = std::move(result);

    uintptr_t context = insertRegistration(registration);
    const char *error = nullptr;
    // xValue and xInverse go together: with both, the aggregate can run over a
    // moving window frame.
    turso_status_code_t status = turso_connection_register_aggregate_function_ptr(
        conn,
        name.c_str(),
        argc,
        flags,
        context,
        tursoUdfAggregateInit,
        tursoUdfAggregateStep,
        tursoUdfAggregateResult,
        isWindow ? tursoUdfAggregateResult : nullptr,
        isWindow ? tursoUdfAggregateInverse : nullptr,
        tursoUdfDestroyContext,
        tursoUdfDestroyAggregate,
        tursoUdfDestroyValue,
        &error);

    if (status != TURSO_OK)
    {
        eraseRegistration(context);
        throw jsi::JSError(rt, error ? error : "failed to register the aggregate");
    }
}

void udfUnregister(jsi::Runtime &rt, turso_connection_t *conn, const std::string &name)
{
    const char *error = nullptr;
    turso_status_code_t status = turso_connection_unregister_function(conn, name.c_str(), &error);
    if (status != TURSO_OK)
    {
        throw jsi::JSError(rt, error ? error : "failed to unregister the function");
    }
}

} // namespace turso
