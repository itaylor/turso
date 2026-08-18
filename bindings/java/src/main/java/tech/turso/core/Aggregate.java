package tech.turso.core;

import tech.turso.annotations.Nullable;

/**
 * A user-defined aggregate function, with one accumulator per group. Values map to SQL as for
 * {@link ScalarFunction}. Extend {@link WindowFunction} to allow {@code OVER}.
 *
 * @param <T> the accumulator type
 */
public abstract class Aggregate<T> {

  /** Makes the accumulator for one group, before any {@link #step}. */
  @Nullable
  public abstract T init();

  /** Folds one row in, returning the accumulator to use from now on. */
  @Nullable
  public abstract T step(@Nullable T state, Object[] args);

  /** The group's answer. Called once per group, even for a group with no rows. */
  @Nullable
  public abstract Object result(@Nullable T state);
}
