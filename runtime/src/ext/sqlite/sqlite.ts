// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

type SQLInputValue = null | number | bigint | string | Uint8Array;
type SQLOutputValue = null | number | bigint | string | Uint8Array;
type SupportedValueType = null | number | bigint | string | Uint8Array;

interface DatabaseSyncOptions {
  readonly open?: boolean;
  readonly readOnly?: boolean;
  readonly allowExtension?: boolean;
  readonly enableForeignKeyConstraints?: boolean;
  readonly enableDoubleQuotedStringLiterals?: boolean;
}

interface FunctionOptions {
  readonly varargs?: boolean;
  readonly deterministic?: boolean;
  readonly directOnly?: boolean;
  readonly useBigIntArguments?: boolean;
}

interface StatementResultingChanges {
  readonly changes: number;
  readonly lastInsertRowid: number | bigint;
}

interface ApplyChangesetOptions {
  readonly filter?: (tableName: string) => boolean;
  readonly onConflict?: number;
}

interface CreateSessionOptions {
  readonly db?: string;
  readonly table?: string;
}

interface Session {
  changeset(): Uint8Array;
  patchset(): Uint8Array;
  close(): void;
}

type SqliteFunction = (...args: SQLInputValue[]) => SQLOutputValue;

const constants = {
  SQLITE_CHANGESET_ABORT: 2,
  SQLITE_CHANGESET_CONFLICT: 3,
  SQLITE_CHANGESET_DATA: 4,
  SQLITE_CHANGESET_FOREIGN_KEY: 5,
  SQLITE_CHANGESET_NOTFOUND: 1,
  SQLITE_CHANGESET_OMIT: 0,
  SQLITE_CHANGESET_REPLACE: 1,
} as const;

class DatabaseSync {
  #dbId: number;

  constructor(filename: string, options?: DatabaseSyncOptions) {
    this.#dbId = sqlite_database_sync_constructor(filename, options);
  }

  // TODO: Implement applyChangeset with proper session extension support
  applyChangeset(
    _changeset: Uint8Array,
    _options?: ApplyChangesetOptions,
  ): void {
    // For now, throw an error indicating this is not yet implemented
    // Full implementation would require SQLite session extension integration
    throw new Error(
      "applyChangeset is not yet implemented - requires session extension support",
    );
  }

  close(): void {
    sqlite_database_sync_close(this.#dbId);
  }

  // TODO: Implement createSession with proper session extension support
  createSession(_options?: CreateSessionOptions): Session {
    // For now, throw an error indicating this is not yet implemented
    // Full implementation would require SQLite session extension integration
    throw new Error(
      "createSession is not yet implemented - requires session extension support",
    );
  }

  enableLoadExtension(enabled: boolean): void {
    sqlite_database_sync_enable_load_extension(this.#dbId, enabled);
  }

  exec(sql: string): void {
    sqlite_database_sync_exec(this.#dbId, sql);
  }

  function(name: string, fn: SqliteFunction, options?: FunctionOptions): void {
    sqlite_database_sync_function(this.#dbId, name, fn, options);
  }

  loadExtension(path: string, entryPoint?: string): void {
    sqlite_database_sync_load_extension(this.#dbId, path, entryPoint);
  }

  open(filename: string, options?: DatabaseSyncOptions): void {
    sqlite_database_sync_open(this.#dbId, filename, options);
  }

  prepare(sql: string): StatementSync {
    const stmtId = sqlite_database_sync_prepare(this.#dbId, sql);
    return new StatementSync(stmtId, this.#dbId);
  }
}

class StatementSync {
  #stmtId: number;
  #dbId: number;

  constructor(stmtId: number, dbId: number) {
    this.#stmtId = stmtId;
    this.#dbId = dbId;
  }

  all(...params: SQLInputValue[]): unknown[] {
    const result = sqlite_statement_sync_all(
      this.#dbId,
      this.#stmtId,
      ...params,
    );

    try {
      if (Array.isArray(result)) {
        return result.map((item) => {
          try {
            return JSON.parse(item as string);
          } catch {
            return item;
          }
        });
      }
      return [];
    } catch {
      return [];
    }
  }

  get expandedSQL(): string {
    return sqlite_statement_sync_expanded_sql(this.#stmtId);
  }

  get(...params: SQLInputValue[]): unknown {
    const result = sqlite_statement_sync_get(
      this.#dbId,
      this.#stmtId,
      ...params,
    );

    try {
      return typeof result === "string" ? JSON.parse(result) : result;
    } catch {
      return result;
    }
  }

  *iterate(...params: SQLInputValue[]): IterableIterator<unknown> {
    const results = sqlite_statement_sync_iterate(
      this.#dbId,
      this.#stmtId,
      ...params,
    );
    for (const result of results) {
      yield result;
    }
  }

  run(...params: SQLInputValue[]): StatementResultingChanges {
    const result = sqlite_statement_sync_run(
      this.#dbId,
      this.#stmtId,
      ...params,
    );

    if (
      result && typeof result === "object" && "changes" in result &&
      "lastInsertRowid" in result
    ) {
      return {
        changes: result.changes as number,
        lastInsertRowid: result.lastInsertRowid as number | bigint,
      };
    }

    try {
      const parsed = JSON.parse(result as string);
      return {
        changes: parsed.changes,
        lastInsertRowid: parsed.lastInsertRowid,
      };
    } catch {
      return { changes: 0, lastInsertRowid: 0 };
    }
  }

  setAllowBareNamedParameters(allowBare: boolean): this {
    sqlite_statement_sync_set_allow_bare_named_parameters(
      this.#stmtId,
      allowBare,
    );
    return this;
  }

  setReadBigInts(readBigInts: boolean): this {
    sqlite_statement_sync_set_read_bigints(this.#stmtId, readBigInts);
    return this;
  }

  get sourceSQL(): string {
    return sqlite_statement_sync_source_sql(this.#stmtId);
  }

  finalize(): void {
    sqlite_statement_sync_finalize(this.#stmtId);
  }
}

(globalThis as unknown as Record<string, unknown>).DatabaseSync = DatabaseSync;
(globalThis as unknown as Record<string, unknown>).StatementSync =
  StatementSync;

(globalThis as unknown as Record<string, unknown>).Database = DatabaseSync;

(globalThis as unknown as Record<string, unknown>).sqlite = {
  DatabaseSync,
  StatementSync,
  constants,
  Database: DatabaseSync,
};
