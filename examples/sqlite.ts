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
  console.log(`${user.name} (${user.age}): ${user.email}`);
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
console.log("\nUsers aged 25-30 with 'o' in name:");
filteredUsers.forEach((user: any) => {
  console.log(`${user.name} (${user.age})`);
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
console.log("\nNULL parameter test results:");
nullResults.forEach((row: any) => {
  console.log(
    `ID: ${row.id}, Value: ${row.value === null ? "NULL" : row.value}`,
  );
});

db.close();
