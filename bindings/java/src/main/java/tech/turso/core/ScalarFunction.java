package tech.turso.core;

import tech.turso.annotations.Nullable;

/** A user-defined scalar function, called once per row. */
public interface ScalarFunction {

  /**
   * Arguments arrive as {@code null}, {@link Long}, {@link Double}, {@link String} or {@code
   * byte[]}; the return value must be one of those, or a boxed number or {@link Boolean}.
   */
  @Nullable
  Object call(Object[] args);
}
