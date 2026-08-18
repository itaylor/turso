import type { NativeConnection, SQLiteValue } from './types';

/** SQLite's flag bits, as the registration entry points take them. */
const FLAG_DETERMINISTIC = 0x800;
const FLAG_DIRECTONLY = 0x80000;

function flagsOf(options: any): number {
  let flags = 0;
  if (options.deterministic) {
    flags |= FLAG_DETERMINISTIC;
  }
  if (options.directOnly) {
    flags |= FLAG_DIRECTONLY;
  }
  return flags;
}

/** The limit better-sqlite3 enforces. */
const MAX_ARGUMENT_COUNT = 100;

/** `undefined` means SQL NULL. */
export type UserFunctionResult = SQLiteValue | boolean | undefined | void;

export type UserFunction = (...args: any[]) => UserFunctionResult;

export interface FunctionOptions {
  /** The same arguments always produce the same result. */
  deterministic?: boolean;
  /** Accept any number of arguments instead of `fn.length`. */
  varargs?: boolean;
  /** Refuse to run from a trigger, view, CHECK constraint or index expression. */
  directOnly?: boolean;
}

export interface AggregateOptions extends FunctionOptions {
  /** The initial accumulator, or a function returning a fresh one per group. */
  start?: any;
  /** Folds one row into the accumulator and returns the new one (or `undefined` to keep it). */
  step: (total: any, ...args: any[]) => any;
  /** Removes a row that left the window frame. Providing it makes the aggregate usable with `OVER`. */
  inverse?: (total: any, ...args: any[]) => any;
  /** Turns the final accumulator into the value SQL sees. Defaults to the accumulator itself. */
  result?: (total: any) => UserFunctionResult;
}

function declaredLength(fn: Function): number {
  const length = fn.length;
  if (!Number.isInteger(length) || length < 0) {
    throw new TypeError('Expected function.length to be a positive integer');
  }
  return length;
}

function checkArgumentCount(argCount: number): number {
  if (argCount > MAX_ARGUMENT_COUNT) {
    throw new RangeError(
      `User-defined functions cannot have more than ${MAX_ARGUMENT_COUNT} arguments`
    );
  }
  return argCount;
}

function functionOption(options: any, key: string, required: boolean): Function | null {
  const value = key in options ? options[key] : null;
  if (typeof value === 'function') {
    return value;
  }
  if (value != null) {
    throw new TypeError(`Expected the "${key}" option to be a function`);
  }
  if (required) {
    throw new TypeError(`Missing required option "${key}"`);
  }
  return null;
}

function checkUnsupportedOptions(options: any): void {
  if ('safeIntegers' in options) {
    throw new TypeError(
      'The "safeIntegers" option is not supported: INTEGER values are always JavaScript numbers in the React Native binding'
    );
  }
}

function checkName(name: any, options: any): void {
  if (typeof name !== 'string') {
    throw new TypeError('Expected first argument to be a string');
  }
  if (!name) {
    throw new TypeError('User-defined function name cannot be an empty string');
  }
  if (typeof options !== 'object' || options === null) {
    throw new TypeError('Expected the options to be an object');
  }
}

/** `options` is optional, so `fn` may arrive in its place. */
export function registerScalarFunction(
  connection: NativeConnection,
  name: any,
  options: any,
  fn: any
): void {
  if (typeof options === 'function') {
    fn = options;
    options = {};
  }
  if (options == null) {
    options = {};
  }

  checkName(name, options);
  if (typeof fn !== 'function') {
    throw new TypeError('Expected last argument to be a function');
  }
  checkUnsupportedOptions(options);

  const argCount = options.varargs ? -1 : checkArgumentCount(declaredLength(fn));
  connection.registerScalarFunction(name, argCount, flagsOf(options), fn);
}

export function registerAggregateFunction(
  connection: NativeConnection,
  name: any,
  options: any
): void {
  if (options == null) {
    options = {};
  }

  checkName(name, options);
  checkUnsupportedOptions(options);

  const start = 'start' in options ? options.start : null;
  const step = functionOption(options, 'step', true)!;
  const inverse = functionOption(options, 'inverse', false);
  const result = functionOption(options, 'result', false);

  // `step` and `inverse` take the running total ahead of the SQL arguments.
  let argCount = -1;
  if (!options.varargs) {
    const declared =
      Math.max(declaredLength(step), inverse ? declaredLength(inverse) : 0) - 1;
    argCount = checkArgumentCount(Math.max(declared, 0));
  }

  // The native side always asks a function for a fresh accumulator.
  const startFn: () => any = typeof start === 'function' ? start : () => start;

  connection.registerAggregateFunction(
    name,
    argCount,
    flagsOf(options),
    startFn,
    step,
    inverse,
    result
  );
}
