package tech.turso.core;

/** Flags a user-defined function may declare, with SQLite's values. Combine with {@code |}. */
public final class TursoFunction {

  /** The same arguments always produce the same answer, so the answer may be reused. */
  public static final int DETERMINISTIC = 0x00000800;

  /** Callable only from application SQL, never from a trigger, view or CHECK constraint. */
  public static final int DIRECTONLY = 0x00080000;

  /** Safe to call from schema SQL even in an untrusted schema. */
  public static final int INNOCUOUS = 0x00200000;

  private TursoFunction() {}
}
