package tech.turso.core;

import tech.turso.annotations.Nullable;

/**
 * An {@link Aggregate} that can also run over a window frame with {@code OVER}.
 *
 * @param <T> the accumulator type
 */
public abstract class WindowFunction<T> extends Aggregate<T> {

  /** The answer for the rows in the frame so far, leaving the accumulator usable. */
  @Nullable
  public abstract Object value(@Nullable T state);

  /** Takes back a row that left the frame, returning the accumulator to use from now on. */
  @Nullable
  public abstract T inverse(@Nullable T state, Object[] args);
}
