import { unlinkSync } from "node:fs";
import { expect, test } from 'vitest'
import { Database } from './compat.js'
import { Database as NativeDatabase } from '#index'

test('classifySql treats reindex as write', () => {
    const db = new NativeDatabase(':memory:');
    db.connectSync();
    try {
        expect(db.classifySql('SELECT 1')).toEqual('read');
        expect(db.classifySql('BEGIN')).toEqual('begin');
        expect(db.classifySql('COMMIT')).toEqual('commit');
        expect(db.classifySql('ROLLBACK')).toEqual('rollback');

        expect(db.classifySql('REINDEX')).toEqual('write');
        expect(db.classifySql('REINDEX idx')).toEqual('write');
        expect(db.classifySql('REINDEX main.idx')).toEqual('write');
    } finally {
        db.close();
    }
})

test('insert returning test', () => {
    const db = new Database(':memory:');
    db.prepare(`create table t (x);`).run();
    const x1 = db.prepare(`insert into t values (1), (2) returning x`).get();
    const x2 = db.prepare(`insert into t values (3), (4) returning x`).get();
    expect(x1).toEqual({ x: 1 });
    expect(x2).toEqual({ x: 3 });
    const all = db.prepare(`select * from t`).all();
    expect(all).toEqual([{ x: 1 }, { x: 2 }, { x: 3 }, { x: 4 }])
})

test('in-memory db', () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE t(x)");
    db.exec("INSERT INTO t VALUES (1), (2), (3)");
    const stmt = db.prepare("SELECT * FROM t WHERE x % 2 = ?");
    const rows = stmt.all([1]);
    expect(rows).toEqual([{ x: 1 }, { x: 3 }]);
})

test('exec multiple statements', async () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE t(x); INSERT INTO t VALUES (1); INSERT INTO t VALUES (2)");
    const stmt = db.prepare("SELECT * FROM t");
    const rows = stmt.all();
    expect(rows).toEqual([{ x: 1 }, { x: 2 }]);
})

test('expanded rows collapse duplicate column names like better-sqlite3', () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE role(path TEXT); CREATE TABLE org_unit(path TEXT)");
    db.exec("INSERT INTO role VALUES ('/Employee'); INSERT INTO org_unit VALUES ('/')");

    const stmt = db.prepare("SELECT role.path, org_unit.path FROM role JOIN org_unit");
    const row = stmt.get();

    expect(Object.keys(row)).toEqual(["path"]);
    expect(row.path).toBe("/");
    expect(row[0]).toBe(undefined);
    expect(row[1]).toBe(undefined);
    expect(row).toEqual({ path: "/" });

    expect(stmt.raw(true).get()).toEqual(["/Employee", "/"]);
})

test('readonly-db', () => {
    const path = `test-${(Math.random() * 10000) | 0}.db`;
    try {
        {
            const rw = new Database(path);
            rw.exec("CREATE TABLE t(x)");
            rw.exec("INSERT INTO t VALUES (1)");
            rw.close();
        }
        {
            const ro = new Database(path, { readonly: true });
            expect(() => ro.exec("INSERT INTO t VALUES (2)")).toThrowError(/Resource is read-only/g);
            expect(ro.prepare("SELECT * FROM t").all()).toEqual([{ x: 1 }])
            ro.close();
        }
    } finally {
        unlinkSync(path);
        unlinkSync(`${path}-wal`);
    }
})

test('file-must-exist', () => {
    const path = `test-${(Math.random() * 10000) | 0}.db`;
    expect(() => new Database(path, { fileMustExist: true })).toThrowError(/failed to open database/);
})

test('on-disk db', () => {
    const path = `test-${(Math.random() * 10000) | 0}.db`;
    try {
        const db1 = new Database(path);
        db1.exec("CREATE TABLE t(x)");
        db1.exec("INSERT INTO t VALUES (1), (2), (3)");
        const stmt1 = db1.prepare("SELECT * FROM t WHERE x % 2 = ?");
        expect(stmt1.columns()).toEqual([{ name: "x", column: null, database: null, table: null, type: null }]);
        const rows1 = stmt1.all([1]);
        expect(rows1).toEqual([{ x: 1 }, { x: 3 }]);
        db1.close();

        const db2 = new Database(path);
        const stmt2 = db2.prepare("SELECT * FROM t WHERE x % 2 = ?");
        expect(stmt2.columns()).toEqual([{ name: "x", column: null, database: null, table: null, type: null }]);
        const rows2 = stmt2.all([1]);
        expect(rows2).toEqual([{ x: 1 }, { x: 3 }]);
        db2.close();
    } finally {
        unlinkSync(path);
        unlinkSync(`${path}-wal`);
    }
})

test('attach', () => {
    const path1 = `test-${(Math.random() * 10000) | 0}.db`;
    const path2 = `test-${(Math.random() * 10000) | 0}.db`;
    try {
        const db1 = new Database(path1, { experimental: ["attach"] });
        db1.exec("CREATE TABLE t(x)");
        db1.exec("INSERT INTO t VALUES (1), (2), (3)");
        const db2 = new Database(path2, { experimental: ["attach"] });
        db2.exec("CREATE TABLE q(x)");
        db2.exec("INSERT INTO q VALUES (4), (5), (6)");

        db1.exec(`ATTACH '${path2}' as secondary`);

        const stmt = db1.prepare("SELECT * FROM t UNION ALL SELECT * FROM secondary.q");
        expect(stmt.columns()).toEqual([{ name: "x", column: null, database: null, table: null, type: null }]);
        const rows = stmt.all([1]);
        expect(rows).toEqual([{ x: 1 }, { x: 2 }, { x: 3 }, { x: 4 }, { x: 5 }, { x: 6 }]);
    } finally {
        unlinkSync(path1);
        unlinkSync(`${path1}-wal`);
        unlinkSync(path2);
        unlinkSync(`${path2}-wal`);
    }
})

test('blobs', () => {
    const db = new Database(":memory:");
    const rows = db.prepare("SELECT x'1020' as x").all();
    expect(rows).toEqual([{ x: Buffer.from([16, 32]) }])
})

test('encryption', () => {
    const path = `test-encryption-${(Math.random() * 10000) | 0}.db`;
    const hexkey = 'b1bbfda4f589dc9daaf004fe21111e00dc00c98237102f5c7002a5669fc76327';
    const wrongKey = 'aaaaaaa4f589dc9daaf004fe21111e00dc00c98237102f5c7002a5669fc76327';
    try {
        const db = new Database(path, {
            encryption: { cipher: 'aegis256', hexkey }
        });
        db.exec("CREATE TABLE t(x)");
        db.exec("INSERT INTO t SELECT 'secret' FROM generate_series(1, 1024)");
        db.exec("PRAGMA wal_checkpoint(truncate)");
        db.close();

        // lets re-open with the same key
        const db2 = new Database(path, {
            encryption: { cipher: 'aegis256', hexkey }
        });
        const rows = db2.prepare("SELECT COUNT(*) as cnt FROM t").all();
        expect(rows).toEqual([{ cnt: 1024 }]);
        db2.close();

        // opening with wrong key MUST fail
        expect(() => {
            const db3 = new Database(path, {
                encryption: { cipher: 'aegis256', hexkey: wrongKey }
            });
            db3.prepare("SELECT * FROM t").all();
        }).toThrow();

        // opening without encryption MUST fail
        expect(() => {
            const db5 = new Database(path);
            db5.prepare("SELECT * FROM t").all();
        }).toThrow();
    } finally {
        unlinkSync(path);
    }
})

test('user-defined scalar function', () => {
    const db = new Database(":memory:");
    expect(db.function('add2', (a, b) => a + b)).toBe(db);
    expect(db.prepare("SELECT add2(2, 3) AS v").get()).toEqual({ v: 5 });

    expect(() => db.prepare("SELECT add2(1)").get()).toThrow(/wrong number of arguments to function add2/);

    db.function('count_args', { varargs: true }, (...args) => args.length);
    expect(db.prepare("SELECT count_args() AS v").get()).toEqual({ v: 0 });
    expect(db.prepare("SELECT count_args(1, 2, 3) AS v").get()).toEqual({ v: 3 });
})

test('user-defined function argument types', () => {
    const db = new Database(":memory:");
    db.function('describe', (x) => `${typeof x}:${x}`);
    expect(db.prepare("SELECT describe(NULL) AS v").get()).toEqual({ v: 'object:null' });
    expect(db.prepare("SELECT describe(7) AS v").get()).toEqual({ v: 'number:7' });
    expect(db.prepare("SELECT describe(1.5) AS v").get()).toEqual({ v: 'number:1.5' });
    expect(db.prepare("SELECT describe('hi') AS v").get()).toEqual({ v: 'string:hi' });

    db.function('blob_length', (x) => x.length);
    expect(db.prepare("SELECT blob_length(x'102030') AS v").get()).toEqual({ v: 3 });
})

test('user-defined function return types', () => {
    const db = new Database(":memory:");
    const returns = {
        null: null,
        undefined: undefined,
        text: 'hi',
        integer: 42,
        real: 1.5,
        bigint: 10n,
        blob: Buffer.from([1, 2]),
        boolean: true,
    };
    db.function('ret', (kind) => returns[kind]);

    expect(db.prepare("SELECT typeof(ret('null')) AS t, ret('null') AS v").get()).toEqual({ t: 'null', v: null });
    expect(db.prepare("SELECT typeof(ret('undefined')) AS t, ret('undefined') AS v").get()).toEqual({ t: 'null', v: null });
    expect(db.prepare("SELECT typeof(ret('text')) AS t, ret('text') AS v").get()).toEqual({ t: 'text', v: 'hi' });
    expect(db.prepare("SELECT typeof(ret('integer')) AS t, ret('integer') AS v").get()).toEqual({ t: 'integer', v: 42 });
    expect(db.prepare("SELECT typeof(ret('real')) AS t, ret('real') AS v").get()).toEqual({ t: 'real', v: 1.5 });
    expect(db.prepare("SELECT typeof(ret('bigint')) AS t, ret('bigint') AS v").get()).toEqual({ t: 'integer', v: 10 });
    expect(db.prepare("SELECT typeof(ret('blob')) AS t, ret('blob') AS v").get()).toEqual({ t: 'blob', v: Buffer.from([1, 2]) });
    expect(db.prepare("SELECT typeof(ret('boolean')) AS t, ret('boolean') AS v").get()).toEqual({ t: 'integer', v: 1 });
})

test('a user-defined function returning a value SQL has no type for throws a TypeError', () => {
    const db = new Database(":memory:");
    db.function('bad', () => ({ a: 1 }));
    expect(() => db.prepare("SELECT bad()").get()).toThrow(TypeError);
    expect(() => db.prepare("SELECT bad()").get()).toThrow(/user-defined function bad\(\) returned an invalid value/);
})

test('a throwing user-defined function fails the statement with the error it threw', () => {
    const db = new Database(":memory:");
    db.function('boom', () => { throw new RangeError("kaboom"); });
    expect(() => db.prepare("SELECT boom()").get()).toThrow(RangeError);
    expect(() => db.prepare("SELECT boom()").get()).toThrow(/kaboom/);
    expect(db.prepare("SELECT 1 AS v").get()).toEqual({ v: 1 });
})

test('user-defined function options', () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE t(x)");
    db.exec("INSERT INTO t VALUES (1), (2), (3)");

    let deterministicCalls = 0;
    db.function('det', { deterministic: true }, (x) => { deterministicCalls++; return x; });
    db.prepare("SELECT * FROM t WHERE x = det(2)").all();
    expect(deterministicCalls).toBe(1);

    let plainCalls = 0;
    db.function('plain', (x) => { plainCalls++; return x; });
    db.prepare("SELECT * FROM t WHERE x = plain(2)").all();
    expect(plainCalls).toBe(3);

    // directOnly: fine from top-level SQL, refused from schema SQL.
    db.function('direct', { directOnly: true }, () => 1);
    expect(db.prepare("SELECT direct() AS v").get()).toEqual({ v: 1 });
    db.exec("CREATE TABLE d(x CHECK (direct() = 1))");
    expect(() => db.exec("INSERT INTO d VALUES (1)")).toThrow(/unsafe use of direct\(\)/);
})

test('closing the database from inside a user-defined function fails cleanly', () => {
    const db = new Database(":memory:");
    db.function('closer', () => { db.close(); return 1; });
    expect(() => db.prepare("SELECT closer() AS v").get()).toThrow(/busy executing a query/);
    expect(db.prepare("SELECT 1 AS v").get()).toEqual({ v: 1 });
    db.close();
})

test('user-defined function safeIntegers', () => {
    const db = new Database(":memory:");
    db.function('kind', (x) => typeof x);
    db.function('safe_kind', { safeIntegers: true }, (x) => typeof x);
    db.function('unsafe_kind', { safeIntegers: false }, (x) => typeof x);

    expect(db.prepare("SELECT kind(5) AS v").get()).toEqual({ v: 'number' });
    expect(db.prepare("SELECT safe_kind(5) AS v").get()).toEqual({ v: 'bigint' });

    db.defaultSafeIntegers(true);
    expect(db.prepare("SELECT kind(5) AS v").get()).toEqual({ v: 'bigint' });
    expect(db.prepare("SELECT unsafe_kind(5) AS v").get()).toEqual({ v: 'number' });
})

test('user-defined function argument validation', () => {
    const db = new Database(":memory:");
    expect(() => (db as any).function(42, () => 1)).toThrow(TypeError);
    expect(() => (db as any).function('')).toThrow(TypeError);
    expect(() => (db as any).function('x')).toThrow(TypeError);
    expect(() => (db as any).function('x', {}, 'not a function')).toThrow(TypeError);
    expect(() => (db as any).function('x', 'not options', () => 1)).toThrow(TypeError);
    expect(() => (db as any).function('x', (...args) => args.length)).not.toThrow();
})

test('a user-defined function shadows a built-in of the same name', () => {
    const db = new Database(":memory:");
    expect(db.prepare("SELECT abs(-1) AS v").get()).toEqual({ v: 1 });
    db.function('abs', (x) => 'shadowed');
    expect(db.prepare("SELECT abs(-1) AS v").get()).toEqual({ v: 'shadowed' });
})

test('a user-defined function can query the database', () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE t(x)");
    db.exec("INSERT INTO t VALUES (1), (2), (3)");
    db.function('row_count', () => db.prepare("SELECT COUNT(*) AS c FROM t").get().c);
    expect(db.prepare("SELECT row_count() AS v").get()).toEqual({ v: 3 });
})

test('a user-defined function cannot re-enter the statement that called it', () => {
    const db = new Database(":memory:");
    let stmt;
    db.function('reenter', () => {
        try {
            stmt.get();
            return 'ran';
        } catch (err) {
            return String(err.message);
        }
    });
    stmt = db.prepare("SELECT reenter() AS v");
    expect(stmt.get()).toEqual({ v: 'statement is already running' });
})

test('user-defined aggregate', () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE t(x)");
    db.exec("INSERT INTO t VALUES (1), (2), (3)");

    expect(db.aggregate('mysum', { start: 0, step: (total, x) => total + x })).toBe(db);
    expect(db.prepare("SELECT mysum(x) AS v FROM t").get()).toEqual({ v: 6 });

    expect(db.prepare("SELECT mysum(x) AS v FROM t WHERE x > 100").get()).toEqual({ v: 0 });

    db.aggregate('keep', { start: 7, step: (total, x) => { } });
    expect(db.prepare("SELECT keep(x) AS v FROM t").get()).toEqual({ v: 7 });
})

test('user-defined aggregate start option can be a function', () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE t(g, x)");
    db.exec("INSERT INTO t VALUES ('a', 1), ('a', 2), ('b', 3)");
    db.aggregate('collect', {
        start: () => [],
        step: (total, x) => { total.push(x); return total; },
        result: (total) => total.join(','),
    });
    expect(db.prepare("SELECT g, collect(x) AS v FROM t GROUP BY g ORDER BY g").all())
        .toEqual([{ g: 'a', v: '1,2' }, { g: 'b', v: '3' }]);
})

test('user-defined aggregate works as a window function when it has an inverse', () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE t(x)");
    db.exec("INSERT INTO t VALUES (1), (2), (3), (4)");
    db.aggregate('wsum', {
        start: 0,
        step: (total, x) => total + x,
        inverse: (total, x) => total - x,
    });
    const rows = db.prepare(
        "SELECT x, wsum(x) OVER (ORDER BY x ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) AS v FROM t ORDER BY x",
    ).all();
    expect(rows).toEqual([{ x: 1, v: 1 }, { x: 2, v: 3 }, { x: 3, v: 5 }, { x: 4, v: 7 }]);
})

test('user-defined aggregate without an inverse cannot be a window function', () => {
    const db = new Database(":memory:");
    db.exec("CREATE TABLE t(x)");
    db.exec("INSERT INTO t VALUES (1), (2)");
    db.aggregate('nosum', { start: 0, step: (total, x) => total + x });
    expect(() => db.prepare("SELECT nosum(x) OVER () FROM t").all())
        .toThrow(/nosum\(\) may not be used as a window function/);
})

test('user-defined aggregate argument validation', () => {
    const db = new Database(":memory:");
    expect(() => (db as any).aggregate('x', {})).toThrow(/Missing required option "step"/);
    expect(() => (db as any).aggregate('x', { step: 'nope' })).toThrow(/Expected the "step" option to be a function/);
    expect(() => (db as any).aggregate('x', { step: (t, x) => t, inverse: 5 })).toThrow(/Expected the "inverse" option to be a function/);
    expect(() => (db as any).aggregate('x', { step: (t, x) => t, result: 5 })).toThrow(/Expected the "result" option to be a function/);
    expect(() => (db as any).aggregate('', { step: (t, x) => t })).toThrow(TypeError);
    expect(() => (db as any).aggregate(5, { step: (t, x) => t })).toThrow(TypeError);
})
