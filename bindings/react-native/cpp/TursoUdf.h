#pragma once

#include <jsi/jsi.h>
#include <ReactCommon/CallInvoker.h>
#include <cstdint>
#include <memory>
#include <string>

extern "C" {
    struct turso_connection;
    typedef struct turso_connection turso_connection_t;
}

namespace turso {

using namespace facebook;

/**
 * Statements are stepped synchronously inside a JSI host function, so the
 * engine calls a user-defined function on the JS thread while it is busy in
 * that step: the callbacks call the stored jsi::Function directly, since
 * hopping through the CallInvoker would deadlock. The CallInvoker is only for
 * cleanup, because JSI values may only be destroyed on the JS thread.
 */
void udfSetCallInvoker(const std::shared_ptr<react::CallInvoker> &invoker);

void udfInvalidate();

/**
 * `argc` is the argument count, or -1 for any number; `flags` uses SQLite's bit
 * values (0x800 DETERMINISTIC, 0x80000 DIRECTONLY).
 */
void udfRegisterScalar(
    jsi::Runtime &rt,
    turso_connection_t *conn,
    const std::string &name,
    int32_t argc,
    uint32_t flags,
    std::shared_ptr<jsi::Function> fn);

/** `start` makes a fresh accumulator per group; `inverse` allows use with `OVER`. */
void udfRegisterAggregate(
    jsi::Runtime &rt,
    turso_connection_t *conn,
    const std::string &name,
    int32_t argc,
    uint32_t flags,
    std::shared_ptr<jsi::Function> start,
    std::shared_ptr<jsi::Function> step,
    std::shared_ptr<jsi::Function> inverse,
    std::shared_ptr<jsi::Function> result);

/** Remove every registration of `name`, whatever its argument count. */
void udfUnregister(jsi::Runtime &rt, turso_connection_t *conn, const std::string &name);

} // namespace turso
