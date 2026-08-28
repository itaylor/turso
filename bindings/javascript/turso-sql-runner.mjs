#!/usr/bin/env node
/**
 * SQL runner script for the sqltest JavaScript backend.
 * Reads SQL from stdin, executes via @tursodatabase/database, outputs pipe-separated results.
 *
 * Usage: node turso-sql-runner.mjs <database_path> [--readonly]
 *
 * This script expects to be run from the bindings/javascript directory where
 * the @tursodatabase/database package is available.
 *
 * Known limitations:
 * - JavaScript's number type doesn't distinguish between 1 and 1.0, so float
 *   formatting may differ from the Rust backend for whole-number floats.
 * - Very large integers (exceeding i64) may have precision loss as JavaScript
 *   numbers are IEEE 754 doubles with 53 bits of mantissa precision.
 */

import { pathToFileURL } from 'node:url';
import { splitStatements } from './turso-sql-split.mjs';

async function readStdin() {
    const chunks = [];
    for await (const chunk of process.stdin) {
        chunks.push(chunk);
    }
    return Buffer.concat(chunks).toString('utf-8');
}

function formatValue(value) {
    if (value === null || value === undefined) {
        return '';
    }
    if (typeof value === 'bigint') {
        return value.toString();
    }
    if (typeof value === 'number') {
        // Handle special float values to match SQLite output
        if (value === Infinity) {
            return 'Inf';
        }
        if (value === -Infinity) {
            return '-Inf';
        }
        if (Number.isNaN(value)) {
            return '';  // SQLite returns NULL for NaN
        }
        // For integers, use toString() directly
        if (Number.isInteger(value)) {
            return value.toString();
        }
        // SQLite uses %.15g format (15 significant digits, trailing zeros removed)
        // toPrecision gives significant digits, parseFloat removes trailing zeros
        return parseFloat(value.toPrecision(15)).toString();
    }
    if (value instanceof Uint8Array || Buffer.isBuffer(value)) {
        // Output blob as raw bytes (matches SQLite/Rust backend behavior)
        // This will display as text if the bytes are printable ASCII
        return Buffer.from(value).toString('utf-8');
    }
    return String(value);
}

/**
 * The standard `udf_*` set a `.sqltest` file gets by asking for `@requires udf`.
 * Mirrors `register_udf_functions` in testing/sqltest/src/backends/rust.rs,
 * which describes each function; keep the two in step.
 */
async function registerUdfTestSet(db) {
    // Anything that is not a number, NULL included, reads as 0. Integers
    // arrive as BigInt because the runner turns on defaultSafeIntegers.
    const asInt = (v) => {
        if (typeof v === 'bigint') return v;
        if (typeof v === 'number') return BigInt(Math.trunc(v));
        return 0n;
    };
    const identity = (x) => x;

    // udf_add(a, b): deterministic, NULL-propagating integer add.
    await db.function('udf_add', { deterministic: true }, (a, b) =>
        a === null || b === null ? null : asInt(a) + asInt(b));
    // udf_concat(...): variadic text concat; NULL contributes nothing.
    await db.function('udf_concat', { deterministic: true, varargs: true }, (...args) =>
        args.map(formatValue).join(''));
    // udf_nondet(x): x plus a call counter, so two calls with the same argument differ.
    let nondetCalls = 0n;
    await db.function('udf_nondet', (x) => asInt(x) + (++nondetCalls));
    // udf_direct / udf_innocuous / udf_plain: identity with the matching flag.
    await db.function('udf_direct', { directOnly: true }, identity);
    await db.function('udf_innocuous', { innocuous: true }, identity);
    await db.function('udf_plain', identity);
    // udf_fail(x): always errors.
    await db.function('udf_fail', (_x) => { throw new Error('udf_fail called'); });

    // udf_sum(x): plain aggregate; udf_wsum(x): window-capable; udf_count0(): counts rows.
    await db.aggregate('udf_sum', { start: 0n, step: (total, x) => total + asInt(x) });
    await db.aggregate('udf_wsum', {
        start: 0n,
        step: (total, x) => total + asInt(x),
        inverse: (total, x) => total - asInt(x),
    });
    await db.aggregate('udf_count0', { start: 0n, step: (total) => total + 1n });
}

function formatRow(row) {
    // Row is an array in raw mode
    return row.map(formatValue).join('|');
}

async function main() {
    const args = process.argv.slice(2);
    if (args.length < 1) {
        console.error('Usage: turso-sql-runner.mjs <database_path> [--readonly]');
        process.exit(1);
    }

    const dbPath = args[0];
    const readonly = args.includes('--readonly');

    const sql = await readStdin();
    if (!sql.trim()) {
        process.exit(0);
    }

    let db;
    try {
        const { connect } = await import('@tursodatabase/database');
        db = await connect(dbPath, { readonly, experimental: ['triggers', 'attach', 'generated_columns', 'without_rowid'] });
        // Enable safe integers to preserve precision for large integers
        db.defaultSafeIntegers(true);
        await registerUdfTestSet(db);
    } catch (err) {
        console.error(`Error: ${err.message}`);
        process.exit(1);
    }

    try {
        // Split into individual statements, filtering out comments and empty lines
        const statements = splitStatements(sql);

        // Accumulate results from ALL queries (matches Rust backend behavior)
        const allResults = [];

        for (const stmt of statements) {
            const trimmed = stmt.trim();
            if (!trimmed) continue;

            const prepared = db.prepare(trimmed);
            prepared.raw(true);
            const rows = await prepared.all();
            allResults.push(...rows);
            prepared.close();
        }

        // Output all accumulated results
        for (const row of allResults) {
            console.log(formatRow(row));
        }

    } catch (err) {
        // Output error in a format the test runner can detect
        console.log(`Error: ${err.message}`);
        process.exit(0); // Exit 0 so the error can be captured as output
    } finally {
        if (db) {
            await db.close();
        }
    }
}

const isMain = process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;

if (isMain) {
    main().catch(err => {
        console.error(`Error: ${err.message}`);
        process.exit(1);
    });
}
