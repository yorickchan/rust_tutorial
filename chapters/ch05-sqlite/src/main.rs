// 第 5 章：SQLite 資料庫（rusqlite）
//
// 本程式展示如何用 rusqlite 對記憶體 SQLite 資料庫做 CRUD 操作。
// 注意：rusqlite 是「同步、阻塞」的，對應 Python 的 sqlite3 模組，最直覺。

use rusqlite::{params, Connection};

/// 一筆待辦事項，對應資料表 todos 的一列。
#[derive(Debug)]
#[allow(dead_code)]
struct Todo {
    id: i32,
    title: String,
    done: bool,
}

fn main() -> rusqlite::Result<()> {
    // 開啟記憶體資料庫（對應 Python 的 sqlite3.connect(":memory:")）。
    // 程式結束後資料即消失，這裡為了簡化範例不寫入磁碟。
    let conn = Connection::open_in_memory()?;

    // === 1. 建立資料表 ===
    // done 用 INTEGER（0/1）儲存布林值，因為 SQLite 沒有真正的 BOOL 型別。
    conn.execute(
        "CREATE TABLE todos (id INTEGER PRIMARY KEY, title TEXT NOT NULL, done INTEGER DEFAULT 0)",
        [],
    )?;

    // === 2. 新增（Create）===
    // params![] 巨集產生位置參數，?1 對應第一個參數，可防止 SQL 注入。
    conn.execute("INSERT INTO todos (title) VALUES (?1)", params!["買牛奶"])?;
    conn.execute("INSERT INTO todos (title) VALUES (?1)", params!["寫 Rust"])?;

    println!("--- 新增後查詢 ---");
    // 用一個區塊包住 SELECT，讓 stmt（借用 conn）在區塊結束時被釋放，
    // 之後才能再對 conn 做 UPDATE。否則借用檢查器會報錯：conn 同時被不可變
    // 借用（prepare）與可變借用（execute）。
    {
        let mut stmt = conn.prepare("SELECT id, title, done FROM todos")?;
        let todos = stmt.query_map([], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                // SQLite 存的是 INTEGER，這裡轉回 Rust 的 bool。
                done: row.get::<_, i32>(2)? != 0,
            })
        })?;

        for todo in todos {
            println!("{:?}", todo?);
        }
    }

    // === 3. 更新（Update）===
    // 將編號 1 的待辦標記為完成。此時 conn 的 stmt 借用已在上面的區塊結束時釋放。
    conn.execute("UPDATE todos SET done = 1 WHERE id = ?1", params![1])?;

    println!("--- 更新後查詢 ---");
    // 再次查詢，觀察更新結果。同樣用區塊包住以確保借用正確釋放。
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

    // === Python 開發者注意 ===
    // rusqlite 是「同步阻塞」的。若在 tokio 的 async handler（例如 axum）裡
    // 直接呼叫 rusqlite，會卡住整個 worker thread，導致其他 async task 無法
    // 執行。解決方式有二：
    //   1. 用 tokio::task::spawn_blocking 把 DB 操作搬到阻塞執行緒池。
    //   2. 改用 sqlx（async 資料庫 crate），它還能在編譯期檢查 SQL 語法。
    // 詳見 README.md 的「同步 vs 非同步」小節。

    Ok(())
}
