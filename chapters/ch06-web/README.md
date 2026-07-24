# 第 6 章：Web 開發（axum）

本章是整個教程的壓軸章，整合前面所學的 **async（tokio）**、**serde（JSON 序列化）**，
並用 **axum** 打造一個完整的 Todo REST API。學完本章，你就能用 Rust 寫出與
FastAPI / Flask 同等級的後端服務。

## 學習目標

- 用 axum 建立 REST API，理解 Router 與路由配置
- 理解 Extractor 模式：用型別從請求中「萃取」資料（State、Path、Json）
- 用 `Arc<Mutex<T>>` 管理跨 task 的共用可變狀態
- 整合 async + serde + JSON，完成一個可運作的 CRUD API

## 本章相依套件與 Cargo.toml

本章是壓軸整合章，用到 4 個 crate：`axum`（web 框架）、`tokio`（async runtime）、`serde`（序列化）、`serde_json`（JSON 格式）。完整 `Cargo.toml`：

```toml
[package]
name = "ch06-web"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[[bin]]
name = "ch06-web"
path = "src/main.rs"
```

### 各套件用途與 features 說明

| crate | 用途 | Python 對照 | 為什麼選它 |
|---|---|---|---|
| `axum` | web 框架（路由、extractor、middleware） | `FastAPI` / `Flask` | Tokio 生態的現代 web 框架，型別導向、文件清楚 |
| `tokio` | async runtime（axum 建立在 tokio 上） | `uvicorn` / `asyncio` | axum 是 async 框架，必須有 runtime 才能跑 |
| `serde` | 序列化框架（`Serialize` / `Deserialize` trait） | 無直接對應 | 讓 `Todo` struct 能轉成 JSON 回應、從 JSON 請求還原 |
| `serde_json` | JSON 格式實作 | `json` 模組 | axum 的 `Json` extractor 內部用它 |

### axum 為什麼不用寫 features？

與前面章節不同，`axum = "0.8"` 沒有 `features = [...]`。這是因為 axum 的 **default features 已經包含本章需要的功能**：`http1`（HTTP/1.1 支援）、`json`（JSON extractor/reponse）、`tokio`（整合 runtime）、`query`（query string 解析）、`matched-path`、`original-uri`（路由匹配用）。本章的 CRUD API 只用到這些基礎功能，所以不必額外開 feature。

如果之後要做 WebSocket，才需加 `features = ["ws"]`；要做 HTTP/2 要 `features = ["http2"]`。本章不需要。

### features 開關說明

- **`axum = "0.8"`**：純版本號、用 default features。理由如上--default 已含 http1 + json + tokio。本教程程式碼靠這些就夠。
- **`tokio = { features = ["full"] }`**：全功能 runtime。本章用 `#[tokio::main]` 與 `axum::serve(listener, app).await`，需要 runtime + macros 等 feature。
- **`serde = { features = ["derive"] }`**：啟用 `#[derive(Serialize, Deserialize)]`，讓 `Todo` struct 能在 JSON 與 Rust 之間轉換。詳見 ch00〈什麼是 `features = ["derive"]`？〉小節。
- **`serde_json = "1"`**：純版本號、無 features。axum 的 `Json` extractor 內部會用到，顯式列出方便理解。

### 安裝指令對照

```bash
# 方法一：cargo add（推薦）
cargo add axum
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json

# 方法二：直接編輯 [dependencies] 區塊（如上面的 Cargo.toml 所示）
```

Python 對照：相當於 `pip install fastapi uvicorn`（加上序列化靠 Python 內建）。本章整合了前面所有觀念：async（ch03）、serde 序列化（ch04）、共用狀態用 `Arc<Mutex<T>>`（對應 Python 的 `threading.Lock`）--是整個教程的壓軸。

## Python 對照

如果你寫過 FastAPI 或 Flask，axum 的概念會很熟悉，只是語法更型別導向：

| 概念 | Python（FastAPI / Flask） | Rust（axum） |
|---|---|---|
| 建立路由 | `@app.get("/todos")` | `.route("/todos", get(handler))` |
| 路徑參數 | `def handler(id: int):` | `async fn handler(Path(id): Path<u32>)` |
| 請求 body | `def handler(body: TodoModel):`（Pydantic） | `async fn handler(Json(body): Json<Todo>)`（serde） |
| 回傳 JSON | `return {"message": "hi"}` | `Json(Resp { message: "hi".into() })` |
| 資料模型 | `class Todo(BaseModel)` | `#[derive(Serialize, Deserialize)] struct Todo` |
| 共用狀態 | `app.state` / 全域 dict | `Arc<Mutex<T>>` + `.with_state()` |
| 啟動伺服器 | `uvicorn.run(app)` | `axum::serve(listener, app).await` |

並排版碼對照：

```python
# Python (FastAPI)
from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI()

class Todo(BaseModel):
    id: int
    title: str
    done: bool = False

@app.get("/todos/{id}")
def get_todo(id: int):
    todo = find_todo(id)
    if todo is None:
        return {"error": "not found"}, 404
    return todo
```

```rust
// Rust (axum)
use axum::{extract::Path, Json, http::StatusCode};

async fn get_todo(
    Path(id): Path<u32>,
) -> Result<Json<Todo>, (StatusCode, &'static str)> {
    match find_todo(id) {
        Some(todo) => Ok(Json(todo)),
        None => Err((StatusCode::NOT_FOUND, "找不到該待辦")),
    }
}
```

## 概念講解

### 1. Router 與方法鏈

axum 用 `Router` 把 URL 路徑對應到 handler 函數：

```rust
let app = Router::new()
    .route("/todos", get(list_todos).post(create_todo))
    .route("/todos/{id}", get(get_todo).put(update_todo).delete(delete_todo))
    .with_state(state);
```

- `.route(path, method(handler))` 註冊一條路由
- 同一路徑可鏈接多個 HTTP 方法：`.get(...).post(...).put(...).delete(...)`
- **axum 0.8 路徑語法**：路徑參數用 `{id}`（大括號），**不是**舊版的 `:id`。
  在 0.8 用 `:id` 會在註冊路由時直接 panic，這是最常見的升級陷阱。

### 2. Extractor（萃取器）

Extractor 是 axum 的核心模式：靠**型別推導**從請求中「萃取」資料。
你在 handler 的參數列宣告你要什麼型別，axum 就自動幫你從請求的對應位置取出來。

| Extractor | 從哪裡取 | 範例 |
|---|---|---|
| `State<T>` | 注入的應用狀態 | `State(s): State<AppState>` |
| `Path<T>` | URL 路徑參數 | `Path(id): Path<u32>` |
| `Json<T>` | request body（反序列化） | `Json(body): Json<CreateTodo>` |

> **Python 開發者注意**：不像 Flask/FastAPI 把 `id` 直接當函數參數
> （`def user(id):`），axum 必須用 `Path(id): Path<u32>` extractor。
> 型別就是契約——axum 看到 `Path<u32>` 就知道要從路徑取參數並轉成 `u32`。

### 3. `Json<T>`：既是 extractor 又是 response

`Json<T>` 是雙向的：
- 作為**參數**（extractor）：把 request body 的 JSON 反序列化成 `T`
- 作為**回傳值**（response）：把 `T` 序列化成 JSON，並自動設定 `Content-Type: application/json`

這對應 Python 中 `return dict`（自動 JSON）與 `body: PydanticModel`（自動解析）兩件事。

### 4. State 與 `.with_state()`

透過 `.with_state(state)` 把共用狀態注入 Router，handler 再用
`State(s): State<AppState>` 取出。

```rust
#[derive(Clone)]                      // <-- State 必須 Clone
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
    next_id: Arc<Mutex<u32>>,
}
```

- **State 必須實作 `Clone`**：axum 內部會複製 state 給每個請求。
- 我們用 `Arc`（原子引用計數指標）包住 `Mutex`，所以 clone 只增加指標計數，
  底層資料仍是同一份——這是 Rust 共用資料的標準模式。

### 5. `Arc<Mutex<T>>`：共用可變狀態

要在多個 async task 間安全地讀寫同一份資料，標準組合是 `Arc<Mutex<T>>`：

- `Arc<T>`：Atomic Reference Counted。多個 task 共用同一份 `T`，引用計數是原子操作（執行緒安全）。
  對應 Python：`asyncio` 單執行緒所以不太需要，但在多執行緒 Python 會用 `multiprocessing.Manager`。
- `Mutex<T>`：互斥鎖。同一時間只允許一個 task 持有鎖並修改 `T`。
  `.lock().await` 取得鎖（async 版本，不阻塞整個執行緒）。
  對應 Python：`asyncio.Lock`。

```rust
let mut todos = s.todos.lock().await;  // 取得鎖
todos.push(new_todo);                   // 安全修改
// 離開作用域時 lock 自動釋放
```

> **鎖的使用紀律**：盡早 `drop(lock)` 釋放鎖，避免持有鎖時 await 其他操作造成死結。
> 本章範例在 `create_todo` 中明確 `drop(next_id)` 後才鎖 `todos`。

### 6. 統一錯誤回傳型別

get / update / delete 三個 handler 都可能回傳 404，我們用
`Result<Json<Todo>, (StatusCode, &'static str)>` 統一回傳型別：

```rust
async fn get_todo(...) -> Result<Json<Todo>, (StatusCode, &'static str)> {
    match todo {
        Some(t) => Ok(Json(t.clone())),
        None => Err((StatusCode::NOT_FOUND, "找不到該待辦")),
    }
}
```

`(StatusCode, &'static str)` 會被 axum 轉成帶狀態碼與純文字 body 的回應。
這比每個 handler 各寫一種回傳型別更一致、好維護。

### 7. 啟動伺服器

```rust
let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
axum::serve(listener, app).await.unwrap();
```

`axum::serve` 是 0.7+ 的伺服器入口（取代舊的 `Server::bind`），
接收一個 `TcpListener` 與 `Router`，跑直到關閉。它在 tokio runtime 內運作，
所以能同時處理多個請求（每個請求是獨立的 task）。

## 程式碼解析

### 資料結構

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    done: Option<bool>,
}
```

- `Todo` 同時 `Serialize`（回應用）與 `Deserialize`（理論上請求也用）。
- `CreateTodo` 只有 `title`——id 與 done 由伺服器產生，不信任客戶端傳的。
- `UpdateTodo` 用 `Option`：只有客戶端有傳的欄位才更新，沒傳的保留原值。
  這對應 REST 的 PATCH 語意（部分更新），我們用在 PUT 上。

### State

```rust
#[derive(Clone)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
    next_id: Arc<Mutex<u32>>,
}
```

Clone 只複製 `Arc`（指標），底層 `Vec` 與 `u32` 是共享的。兩個 `Mutex` 分開，
避免「鎖住 todos 就連 next_id 也不能動」的過度同步。

### list_todos

```rust
async fn list_todos(State(s): State<AppState>) -> Json<Vec<Todo>> {
    let todos = s.todos.lock().await;
    Json(todos.clone())
}
```

鎖住 `todos`，clone 一份回傳。clone 是必要的——我們不能把借用的資料直接回傳，
而盡早釋放鎖讓其他請求能存取。

### create_todo

```rust
async fn create_todo(
    State(s): State<AppState>,
    Json(body): Json<CreateTodo>,
) -> Json<Todo> {
    let mut next_id = s.next_id.lock().await;
    let id = *next_id;
    *next_id += 1;
    drop(next_id); // 釋放 next_id 的鎖

    let todo = Todo { id, title: body.title, done: false };

    let mut todos = s.todos.lock().await;
    todos.push(todo.clone());
    Json(todo)
}
```

兩個獨立的鎖，逐一取得再盡早釋放。`drop(next_id)` 是顯式釋放——
不寫的話鎖會持續到函數結束，但這裡我們之後還要鎖 `todos`，先放掉避免不必要的持有。

### get_todo / update_todo / delete_todo

三個都接收 `Path(id): Path<u32>`，回傳 `Result<..., (StatusCode, &'static str)>`：

- `get_todo`：用 `iter().find()` 找，找不到回 404。
- `update_todo`：用 `iter_mut().find()` 找到後原地修改，只動 `Option::Some` 的欄位。
- `delete_todo`：用 `Vec::retain` 過濾掉該 id，靠「刪前後長度有無變化」判斷成功與否，
  成功回 `204 No Content`。

### Router 與啟動

```rust
let app = Router::new()
    .route("/todos", get(list_todos).post(create_todo))
    .route("/todos/{id}", get(get_todo).put(update_todo).delete(delete_todo))
    .with_state(state);

let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
axum::serve(listener, app).await.unwrap();
```

注意 `/todos/{id}` 用大括號——這是 axum 0.8 的語法，寫成 `:id` 會 panic。

## 執行方式

啟動伺服器：

```bash
cargo run -p ch06-web
```

看到 `伺服器啟動: http://localhost:3000` 後，開另一個 terminal 用 curl 測試：

```bash
# 列出所有待辦（一開始是空的）
curl http://localhost:3000/todos

# 新增一筆
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"學 Rust"}'

# 查看特定待辦
curl http://localhost:3000/todos/1

# 更新（標記完成）
curl -X PUT http://localhost:3000/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"學 Rust","done":true}'

# 刪除
curl -X DELETE http://localhost:3000/todos/1

# 再列一次確認已刪除
curl http://localhost:3000/todos
```

> 資料存在記憶體中，伺服器重啟後清空。課文練習題引導你思考如何結合第 5 章的 SQLite 做持久化。

## 重點回顧

1. **Router + 方法鏈**：`.route(path, get(h1).post(h2))` 在一條路徑上掛多個方法。
2. **Extractor**：靠型別從請求萃取資料——`State<T>`、`Path<T>`、`Json<T>`。不像 Python 把參數直接列在函數簽名。
3. **`Json<T>` 雙向**：作為參數是反序列化（extractor），作為回傳值是序列化（response，自動 Content-Type）。
4. **State 必須 `Clone`**：用 `.with_state()` 注入，handler 用 `State(s): State<T>` 取出。
5. **`Arc<Mutex<T>>`**：Arc 共用、Mutex 安全修改——async 環境管理可變狀態的標準組合。
6. **axum 0.8 路徑語法**：`{id}` 不是 `:id`。寫錯會在路由註冊時 panic。
7. **`axum::serve(listener, app)`**：0.7+ 的伺服器入口，跑在 tokio runtime 內。

## 練習（選做）

1. **查詢過濾**：加一個 `GET /todos?done=true`，用 `axum::extract::Query` 過濾已完成的項目。
   提示：定義 `#[derive(Deserialize)] struct TodoQuery { done: Option<bool> }`，
   handler 加 `Query(q): Query<TodoQuery>` 參數，鎖住 todos 後用 `iter().filter()` 過濾。

2. **持久化**：把記憶體儲存換成第 5 章的 rusqlite。注意 rusqlite 是同步的，
   在 async handler 裡要用 `tokio::task::spawn_blocking` 包住資料庫操作，避免阻塞 runtime。

3. **錯誤處理進階**：把 `(StatusCode, &'static str)` 換成自訂的 `AppError` enum，
   實作 `IntoResponse`，讓不同錯誤回傳不同 JSON 結構（例如 `{"error": "not found", "id": 1}`）。
