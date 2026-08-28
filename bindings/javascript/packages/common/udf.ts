import { NativeUdfOptions } from "./types.js";

export interface FunctionOptions {
  /** The same arguments always produce the same result. */
  deterministic?: boolean;
  /** Accept any number of arguments instead of `fn.length`. */
  varargs?: boolean;
  /** Refuse to run from a trigger, view, CHECK constraint or index expression. */
  directOnly?: boolean;
  /** Safe to run from schema SQL (a view, trigger, CHECK, ...) even when `PRAGMA trusted_schema` is off. */
  innocuous?: boolean;
  /** Pass INTEGER arguments as BigInt. Defaults to the database's `defaultSafeIntegers` setting. */
  safeIntegers?: boolean;
}

export interface AggregateOptions extends FunctionOptions {
  /** The initial value of the accumulator, or a function returning a fresh one for every group. */
  start?: any;
  /** Folds one row into the accumulator and returns the new value (or `undefined` to keep the old one). */
  step: (total: any, ...args: any[]) => any;
  /** Removes a row that left the window frame. Providing it makes the aggregate usable as a window function. */
  inverse?: (total: any, ...args: any[]) => any;
  /** Turns the final accumulator into the value SQL sees. Defaults to the accumulator itself. */
  result?: (total: any) => any;
}

/** The limit better-sqlite3 enforces. */
const MAX_ARGUMENT_COUNT = 100;

function declaredLength(fn: Function): number {
  const length = fn.length;
  if (!Number.isInteger(length) || length < 0) {
    throw new TypeError("Expected function.length to be a positive integer");
  }
  return length;
}

function checkArgumentCount(argCount: number): number {
  if (argCount > MAX_ARGUMENT_COUNT) {
    throw new RangeError(`User-defined functions cannot have more than ${MAX_ARGUMENT_COUNT} arguments`);
  }
  return argCount;
}

function functionOption(options: any, key: string, required: boolean): Function | null {
  const value = key in options ? options[key] : null;
  if (typeof value === "function") return value;
  if (value != null) throw new TypeError(`Expected the "${key}" option to be a function`);
  if (required) throw new TypeError(`Missing required option "${key}"`);
  return null;
}

function nativeOptions(options: FunctionOptions, argCount: number): NativeUdfOptions {
  return {
    argCount,
    deterministic: !!options.deterministic,
    directOnly: !!options.directOnly,
    innocuous: !!options.innocuous,
    // Left out, the native side follows the database's default.
    safeIntegers: "safeIntegers" in options ? !!options.safeIntegers : undefined,
  };
}

export interface ScalarRegistration {
  name: string;
  options: NativeUdfOptions;
  fn: Function;
}

export interface AggregateRegistration {
  name: string;
  options: NativeUdfOptions;
  start: any;
  step: Function;
  inverse: Function | null;
  result: Function | null;
}

/** `options` is optional, so `fn` may arrive in its place. */
export function scalarRegistration(name: any, options: any, fn: any): ScalarRegistration {
  if (typeof options === "function") {
    fn = options;
    options = {};
  }
  if (options == null) options = {};

  if (typeof name !== "string") throw new TypeError("Expected first argument to be a string");
  if (typeof fn !== "function") throw new TypeError("Expected last argument to be a function");
  if (typeof options !== "object") throw new TypeError("Expected second argument to be an options object");
  if (!name) throw new TypeError("User-defined function name cannot be an empty string");

  const argCount = options.varargs ? -1 : checkArgumentCount(declaredLength(fn));
  return { name, options: nativeOptions(options, argCount), fn };
}

export function aggregateRegistration(name: any, options: any): AggregateRegistration {
  if (options == null) options = {};

  if (typeof name !== "string") throw new TypeError("Expected first argument to be a string");
  if (typeof options !== "object") throw new TypeError("Expected second argument to be an options object");
  if (!name) throw new TypeError("User-defined function name cannot be an empty string");

  const start = "start" in options ? options.start : null;
  const step = functionOption(options, "step", true)!;
  const inverse = functionOption(options, "inverse", false);
  const result = functionOption(options, "result", false);

  // `step` and `inverse` take the running total ahead of the SQL arguments.
  let argCount = -1;
  if (!options.varargs) {
    const declared = Math.max(declaredLength(step), inverse ? declaredLength(inverse) : 0) - 1;
    argCount = checkArgumentCount(Math.max(declared, 0));
  }

  return { name, options: nativeOptions(options, argCount), start, step, inverse, result };
}
