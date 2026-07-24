# 第 5 章：SQLite 資料庫（rusqlite）

## 學習目標

- 學會用 `rusqlite` 操作 SQLite 資料庫（新增、查詢、更新）。
- 理解 Rust 存取資料庫的方式，以及與 Python `sqlite3` 模組的對應關係。
- 認識 `params![]` 巨集如何防止 SQL 注入、`query_map` 如何把列轉成 struct。
- 了解同步（rusqlite）與非同步（sqlx）資料庫存取的差異與取捨。

---

## Python 對照

Python 開發者對 `sqlite3` 模組應該不陌生。rusqlite 的 API 設計與它高度對應：

| Python `sqlite3` | Rust `rusqlite` |
|---|---|
| `sqlite3.connect(":memory:")` | `Connection::open_in_memory()?` |
| `cursor = conn.cursor()` | 直接用 `conn`（rusqlite 不用游標物件） |
| `cursor.execute(sql, params)` | `conn.execute(sql, params![...])?` |
| `cursor.fetchall()` | `stmt.query_map([], \|row\| ...)?` |
| `conn.commit()` | 自動提交（rusqlite 預設 autocommit） |
| `row[0]`、`row["name"]` | `row.get(0)?`、`row.get::<_, T>(1)?` |

下面是完整 CRUD 流程的並排對照：

**Python：**

```python
import sqlite3

conn = sqlite3.connect(":memory:")
conn.execute(
    "CREATE TABLE todos (id INTEGER PRIMARY KEY, title TEXT NOT NULL, done INTEGER DEFAULT 0)"
)

# 新增
conn.execute("INSERT INTO todos (title) VALUES (?)", ("買牛奶",))
conn.execute("INSERT INTO todos (title) VALUES (?)", ("寫 Rust",))

# 查詢
for row in conn.execute("SELECT id, title, done FROM todos"):
    print(dict(id=row[0], title=row[1], done=bool(row[2])))

# 更新
conn.execute("UPDATE todos SET done = 1 WHERE id = ?", (1,))
```

**Rust：**

```rust
use rusqlite::{params, Connection};

let conn = Connection::open_in_memory()?;
conn.execute(
    "CREATE TABLE todos (id INTEGER PRIMARY KEY, title TEXT NOT NULL, done INTEGER DEFAULT 0)",
    [],
)?;

// 新增
conn.execute("INSERT INTO todos (title) VALUES (?1)", params!["買牛奶"])?;
conn.execute("INSERT INTO todos (title) VALUES (?1)", params!["寫 Rust"])?;

// 查詢
let mut stmt = conn.prepare("SELECT id, title, done FROM todos")?;
let todos = stmt.query_map([], |row| {
    Ok(Todo {
        id: row.get(0)?,
        title: row.get(1)?,
        done: row.get::<_, i32>(2)? != 0,
    })
})?;
for todo in todos {
    println!("{:?}", todo?);
}

// 更新
conn.execute("UPDATE todos SET done = 1 WHERE id = ?1", params![1])?;
```

---

## 概念講解

### `bundled` feature：免系統安裝 SQLite

本範例的 `Cargo.toml` 寫了：

```toml
rusqlite = { version = "0.40", features = ["bundled"] }
```

`bundled` feature 會把 SQLite 的 C 原始碼一起編譯並靜態連結進你的程式，**不需要**在系統上額外安裝 `libsqlite3`。對初學者最友善：`cargo build` 一鍵完成，沒有環境設定問題。代價是編譯時間稍長，且產出的執行檔會大一點點。

### `params![]` 巨集：位置參數與防 SQL 注入

SQL 語句中用 `?1`、`?2` 標示佔位符，實際值透過 `params![]` 巨集傳入：

```rust
conn.execute("INSERT INTO todos (title) VALUES (?1)", params!["買牛奶"])?;
conn.execute("UPDATE todos SET done = 1 WHERE id = ?1", params![1])?;
```

`params!["買牛奶"]` 會展開成一個 `&[&dyn ToSql]` 切片。關鍵在於值是**綁定**到佔位符，而不是用字串拼接塞進 SQL，因此使用者輸入永遠不會被當成 SQL 語法的一部分，從根本上防止 SQL 注入攻擊。

### `query_map`：把每一列轉成 Rust struct

`prepare` 會先編譯 SQL 語句，回傳一個 `Statement`（借用 `conn`）；`query_map` 則執行語句並對每一列套用一個閉包，把它轉成你定義的 Rust 型別：

```rust
let mut stmt = conn.prepare("SELECT id, title, done FROM todos")?;
let todos = stmt.query_map([], |row| {
    Ok(Todo {
        id: row.get(0)?,
        title: row.get(1)?,
        done: row.get::<_, i32>(2)? != 0,
    })
})?;
```

- `row.get(0)?`：以索引（0 開始）取欄位值，型別由目標欄位推導。
- `row.get::<_, i32>(2)?`：明確指定要取成 `i32`，因為 SQLite 沒有布林型別，存的是 `0`/`1`。
- `query_map` 回傳的是一個迭代器，每個元素是 `Result<Todo>`，所以迭代時用 `todo?` 解開（若某列轉換失敗會在這裡報錯）。

### `?` 運算子：傳播 `rusqlite::Error`

`main` 的回傳型別是 `rusqlite::Result<()>`，所有 DB 呼叫都用 `?` 結尾。一旦任何一步出錯，`?` 會立刻把 `rusqlite::Error` 往外傳，`main` 隨即結束並印出錯誤。這比 Python 的 `try/except` 更輕量：你不需要層層包 try，只要在簽章標好錯誤型別，`?` 會自動處理。

### 同步 vs 非同步

rusqlite 是**同步、阻塞**的：呼叫 `execute` 或 `query_map` 時，該執行緒會一直等到 SQLite 回應才繼續。這對一般腳本或命令列工具完全沒問題（對應 Python 的 `sqlite3`）。

但如果在 **async 環境**（例如第 6 章的 axum web 伺服器）裡直接呼叫 rusqlite，問題就來了：阻塞呼叫會卡住 tokio 的 worker thread，讓同一條執行緒上的其他 async task 無法被排程，等於讓整個伺服器「停格」。解決方式有兩種：

1. **`tokio::task::spawn_blocking`**：把 rusqlite 的同步操作丟到阻塞執行緒池執行，回傳一個 `Future` 給 async 上下文 `await`。rusqlite 本身不變，只是換個執行緒跑。
2. **改用 `sqlx`**：一個原生 async 的 Rust 資料庫 crate，支援 SQLite。它的 `query!` 巨集會在**編譯期**連線資料庫檢查 SQL 語法與欄位型別，是強大但較進階的功能（需要在編譯時提供一個資料庫連線，例如透過 `DATABASE_URL` 環境變數或離線模式快取）。本入門教程不實作 sqlx，但建議深入 web 開發時認識它。

---

## 程式碼解析

以下逐段解析 `src/main.rs`。

### 資料結構

```rust
#[derive(Debug)]
struct Todo {
    id: i32,
    title: String,
    done: bool,
}
```

一個普通 struct 加上 `#[derive(Debug)]`，這樣可以用 `{:?}` 印出內容，方便觀察。`done` 在 Rust 端是 `bool`，但對應到 SQLite 是 `INTEGER`（0/1），轉換在查詢時處理。

### 開啟記憶體資料庫

```rust
let conn = Connection::open_in_memory()?;
```

`open_in_memory()` 建立一個存在於 RAM 的 SQLite 資料庫，程式結束即消失。對應 Python 的 `sqlite3.connect(":memory:")`。用 `?` 傳播可能的 `io::Error`。

### 建立資料表

```rust
conn.execute(
    "CREATE TABLE todos (id INTEGER PRIMARY KEY, title TEXT NOT NULL, done INTEGER DEFAULT 0)",
    [],
)?;
```

`execute` 用於不回傳資料列的語句（CREATE、INSERT、UPDATE、DELETE）。第二個參數 `[]` 表示沒有參數。`done` 欄位用 `INTEGER DEFAULT 0`，預設未完成。

### 新增資料

```rust
conn.execute("INSERT INTO todos (title) VALUES (?1)", params!["買牛奶"])?;
conn.execute("INSERT INTO todos (title) VALUES (?1)", params!["寫 Rust"])?;
```

`?1` 是位置參數，`params!["買牛奶"]` 把字串綁定到它。因為 `id` 是 `INTEGER PRIMARY KEY`，SQLite 會自動遞增，不需手動指定。

### 查詢：借用的區塊限制（重點）

```rust
println!("--- 新增後查詢 ---");
{
    let mut stmt = conn.prepare("SELECT id, title, done FROM todos")?;
    let todos = stmt.query_map([], |row| {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            done: row.get::<_, i32>(2)? != 0,
        })
    })?;

    for todo in todos {
        println!("{:?}", todo?);
    }
}
```

**為什麼要用 `{ }` 區塊包起來？** 因為 `conn.prepare(&)` 會**不可變借用** `conn`，而 `stmt` 活著時這個借用一直存在。如果接著要 `conn.execute(&)`（UPDATE），那是**可變借用**——Rust 的借用規則禁止同時存在不可變與可變借用，編譯器會報錯。

把 SELECT 放在區塊裡，`stmt` 在區塊結束（`}`）時被 drop，不可變借用隨之釋放，之後的 UPDATE 就能合法地可變借用 `conn`。這是 Rust 所有權系統在實務上的典型應用，Python 沒有這個限制（因為沒有編譯期借用檢查）。

### 更新

```rust
conn.execute("UPDATE todos SET done = 1 WHERE id = ?1", params![1])?;
```

把編號 1 的待辦標為完成。這行程式在 SELECT 區塊的 `}` 之後，所以 `conn` 已可被可變借用，沒有衝突。

### 更新後再查詢

```rust
println!("--- 更新後查詢 ---");
{
    let mut stmt = conn.prepare("SELECT id, title, done FROM todos")?;
    let todos = stmt.query_map([], |row| {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            done: row.get::<_, i32>(2)? != 0,
        })
    })?;

    for todo in todos {
        println!("{:?}", todo?);
    }
}

Ok(())
```

同樣的模式再跑一次查詢，觀察 `done` 是否變成 `true`。最後 `Ok(())` 表示 `main` 成功結束。

---

## 執行方式

```sh
cargo run -p ch05-sqlite
```

預期輸出（示意，實際 `done` 在第一次查詢為 `false`，第二次為 `true`）：

```
--- 新增後查詢 ---
Todo { id: 1, title: "買牛奶", done: false }
Todo { id: 2, title: "寫 Rust", done: false }
--- 更新後查詢 ---
Todo { id: 1, title: "買牛奶", done: true }
Todo { id: 2, title: "寫 Rust", done: false }
```

可以觀察到：新增兩筆後查詢，兩筆都未完成；把編號 1 標記完成後再查詢，第一筆的 `done` 變成 `true`。

---

## 重點回顧

- **`Connection::open_in_memory()`** 開啟記憶體資料庫，對應 Python 的 `sqlite3.connect(":memory:")`。
- **`conn.execute(sql, params![...])`** 執行無回傳列的語句（CREATE/INSERT/UPDATE/DELETE）。
- **`conn.prepare(sql)` + `stmt.query_map([], |row| ...)`** 執行查詢並把每列轉成 struct。
- **`params![]` 巨集**以位置參數（`?1`、`?2`）綁定值，從根本防止 SQL 注入，切勿用字串拼接。
- **`?` 運算子**傳播 `rusqlite::Error`，`main` 簽章標為 `rusqlite::Result<()>`。
- **借用與區塊**：`prepare` 借用 `conn`，若之後要 `execute` 更新，需用 `{ }` 讓 `stmt` 先釋放借用——這是 Rust 所有權系統的實務體現。
- **`bundled` feature** 靜態連結 SQLite，免系統安裝，最適合入門。
- **同步特性**：rusqlite 是阻塞的，在 async 環境要用 `spawn_blocking` 或改用 sqlx。

---

## 練習

1. **新增 DELETE 操作**：寫一段程式刪除所有已完成的 todo，例如：
   ```rust
   conn.execute("DELETE FROM todos WHERE done = 1", [])?;
   ```
   執行後再查詢一次，確認已完成的項目被移除。

2. **抽出查詢函數**：把重複的 SELECT 邏輯抽成一個函數 `fn list_todos(conn: &Connection) -> rusqlite::Result<()>`，在新增後、更新後、刪除後各呼叫一次，體驗 Rust 的函數複用。（提示：函數結束時 `stmt` 自動釋放借用，等同區塊的效果。）

3. **持久化到磁碟**：把 `Connection::open_in_memory()` 改成 `Connection::open("todos.db")`，觀察程式重複執行時資料是否保留。思考：每次執行都 `CREATE TABLE` 會發生什麼？（提示：可用 `CREATE TABLE IF NOT EXISTS`。）
