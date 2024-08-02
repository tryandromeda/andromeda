internal_sqlite_execute(
    "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT);",
);
internal_sqlite_execute(
    "INSERT INTO test (name) VALUES ('John');",
);
internal_sqlite_execute(
    "INSERT INTO test (name) VALUES ('John');",
);

internal_sqlite_execute("SELECT * FROM test;");
