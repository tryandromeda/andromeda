// deno-lint-ignore-file no-explicit-any

const db = new Database(":memory:");

db.exec(`
    CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        age INTEGER,
        email TEXT
    );
    
    CREATE TABLE products (
        id INTEGER PRIMARY KEY,
        name TEXT,
        price REAL,
        in_stock INTEGER
    );
  `);

// Insert with parameters
const insertUserStmt = db.prepare(
  "INSERT INTO users (name, age, email) VALUES (?, ?, ?)",
);
insertUserStmt.run("Alice Smith", 25, "alice@example.com");
insertUserStmt.run("Bob Johnson", 30, "bob@example.com");
insertUserStmt.run("Charlie Brown", 28, "charlie@example.com");

// Select with parameters
const usersOver26 = db.prepare("SELECT * FROM users WHERE age > ?").all(26);
console.log("Users over 26:");
usersOver26.forEach((user: any) => {
  console.log(`  ${user.name} (${user.age}): ${user.email}`);
});

// Update with parameters
db.prepare("UPDATE users SET age = ? WHERE name = ?").run(
  26,
  "Alice Smith",
);

// Complex query with multiple parameters
const filteredUsers = db.prepare(
  "SELECT * FROM users WHERE age BETWEEN ? AND ? AND name LIKE ?",
).all(25, 30, "%o%");
console.log("Users aged 25-30 with 'o' in name:");
filteredUsers.forEach((user: any) => {
  console.log(`  ${user.name} (${user.age})`);
});

// Different parameter types
const insertProductStmt = db.prepare(
  "INSERT INTO products (id, name, price, in_stock) VALUES (?, ?, ?, ?)",
);
insertProductStmt.run(1, "Laptop", 999.99, true);
insertProductStmt.run(2, "Mouse", 25.50, false);

db.exec("CREATE TABLE nullable_test (id INTEGER, value TEXT)");
const nullStmt = db.prepare(
  "INSERT INTO nullable_test (id, value) VALUES (?, ?)",
);
nullStmt.run(1, "not null");
nullStmt.run(2, null);

const nullResults = db.prepare("SELECT * FROM nullable_test").all();
nullResults.forEach((row: any) => {
  console.log(
    `  ID: ${row.id}, Value: ${row.value === null ? "NULL" : row.value}`,
  );
});

const optionsStmt = db.prepare("SELECT * FROM users WHERE name = ?");

optionsStmt.setAllowBareNamedParameters(true);
optionsStmt.setAllowBareNamedParameters(false);
optionsStmt.setReadBigInts(true);

optionsStmt.setReadBigInts(false);

db.enableLoadExtension(true);

db.enableLoadExtension(false);

const result = db.function("test_func", () => "hello");
console.log(result);

const fileDb = new Database("/tmp/test_andromeda.db");
fileDb.exec("CREATE TABLE IF NOT EXISTS test (id INTEGER, data TEXT)");
fileDb.exec("INSERT INTO test (id, data) VALUES (1, 'file test')");

const fileResults = fileDb.prepare("SELECT * FROM test").all();
console.log("File database results:");
fileResults.forEach((row: any) => {
  console.log(`  ID: ${row.id}, Data: ${row.data}`);
});

fileDb.close();

db.exec("BEGIN TRANSACTION");
db.exec(
  "INSERT INTO users (name, age, email) VALUES ('Transaction User', 99, 'trans@test.com')",
);

const preCommitCount = db.prepare(
  "SELECT COUNT(*) as count FROM users WHERE age = 99",
).get() as any;
console.log(`  Users with age 99 before commit: ${preCommitCount.count}`);

db.exec("COMMIT");

const postCommitCount = db.prepare(
  "SELECT COUNT(*) as count FROM users WHERE age = 99",
).get() as any;
console.log(`  Users with age 99 after commit: ${postCommitCount.count}`);

db.exec(`
  CREATE TABLE type_test (
    id INTEGER,
    text_val TEXT,
    int_val INTEGER,
    real_val REAL,
    blob_val BLOB,
    bool_val INTEGER
  )
`);

const typeStmt = db.prepare(
  "INSERT INTO type_test (id, text_val, int_val, real_val, bool_val) VALUES (?, ?, ?, ?, ?)",
);

typeStmt.run(1, "test string", 42, 3.14159, 1);
typeStmt.run(2, "", 0, -99.5, 0);
typeStmt.run(3, "unicode: 🚀", 2147483647, 0.0001, 1);

const typeResults = db.prepare("SELECT * FROM type_test ORDER BY id").all();
console.log("Data type test results:");
typeResults.forEach((row: any) => {
  console.log(
    `  ID: ${row.id}, Text: "${row.text_val}", Int: ${row.int_val}, Real: ${row.real_val}, Bool: ${row.bool_val}`,
  );
});

const startTime = Date.now();

db.exec("CREATE TABLE perf_test (id INTEGER, value TEXT)");
const perfStmt = db.prepare("INSERT INTO perf_test (id, value) VALUES (?, ?)");

const iterations = 1000;
for (let i = 0; i < iterations; i++) {
  perfStmt.run(i, `test_value_${i}`);
}

const count = db.prepare("SELECT COUNT(*) as count FROM perf_test")
  .get() as any;
const endTime = Date.now();

console.log(`  Inserted ${iterations} rows in ${endTime - startTime}ms`);
console.log(`  Final count: ${count.count} rows`);

const propStmt = db.prepare("SELECT * FROM users WHERE name = ? AND age = ?");
console.log("  sourceSQL:", propStmt.sourceSQL);
console.log("  expandedSQL:", propStmt.expandedSQL);

const iterStmt = db.prepare("SELECT * FROM users ORDER BY name");
console.log("Iterating through users:");
let count2 = 0;
for (const user of iterStmt.iterate()) {
  console.log(`  ${(user as any).name} (${(user as any).age})`);
  count2++;
  if (count2 >= 3) break; // Limit output
}
console.log("  ✓ Iterator test completed");

db.close();
