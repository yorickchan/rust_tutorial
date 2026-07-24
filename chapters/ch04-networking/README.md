# 第 4 章：網路程式設計（reqwest）

本章示範如何用 Rust 發送 HTTP 請求、接收回應，並把 JSON 剖析成強型別的
struct。我們使用的函式庫是 [`reqwest`](https://docs.rs/reqwest)（HTTP client）
搭配 [`serde`](https://serde.rs)（序列化／反序列化），這也是 Rust 生態中最主流的
組合。

## 學習目標

- 用 `reqwest::Client` 發送 GET 與 POST 請求。
- 用 serde 的 `#[derive(Deserialize)]` 把 JSON 回應剖析成 Rust struct。
- 理解 `Client` 重用（connection pool）、`error_for_status()`、以及
  `resp.json::<T>()` 與 Python `resp.json()` 的關鍵差異。

## 本章相依套件與 Cargo.toml

本章會用到 4 個 crate：`reqwest`（HTTP client）、`tokio`（async runtime）、`serde`（序列化）、`serde_json`（JSON 格式）。完整 `Cargo.toml`：

```toml
[package]
name = "ch04-networking"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.13", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[[bin]]
name = "ch04-networking"
path = "src/main.rs"
```

### 各套件用途與 features 說明

| crate | 用途 | Python 對照 | 為什麼選它 |
|---|---|---|---|
| `reqwest` | HTTP client（發 GET/POST、管理 connection pool） | `requests` / `httpx` | Rust 最受歡迎的 HTTP client，API 風格接近 `requests` |
| `tokio` | async runtime（reqwest 的非同步 API 需要它） | `asyncio` | reqwest 預設是 async，必須有 runtime 才能跑 |
| `serde` | 序列化框架（提供 `Deserialize` trait） | 無直接對應 | 把 JSON 反序列化成 struct 的基礎 |
| `serde_json` | JSON 格式實作 | `json` 模組 | 配合 serde 做具體 JSON 轉換 |

### 為什麼需要 4 個 crate？

Python 一行 `import requests; requests.get(url).json()` 就搞定，背後是 `requests` 把「HTTP + JSON + 字典」全包了。Rust 把這些職責拆開：

1. `reqwest` 只管 HTTP 協定（發請求、收回應、connection pool）。
2. `serde` 只定義「如何序列化」的 trait，不綁定任何格式。
3. `serde_json` 把 serde trait 對應到 JSON 格式（讀寫 JSON 字串）。
4. `tokio` 提供非同步執行環境（reqwest 的非同步 API 需要它）。

看起來很多，但這是 Rust「每個 crate 只做一件事」的哲學。好處是可替換--不想用 JSON 揜成 `serde_yaml` 就好，HTTP 換成 `hyper` 或 `ureq` 也行。

### features 開關說明

- **`reqwest = { features = ["json"] }`**：啟用 JSON 支援。reqwest 預設不綁 serde，加 `json` feature 後才能用 `resp.json::<T>()`（自動把回應體反序列化成 struct）。不加這個 feature，`resp.json()` 會編譯失敗。
- **`tokio = { features = ["full"] }`**：全功能 runtime。理由同 ch03--本章用 `#[tokio::main]` 巨集與非同步執行，需要 `rt-multi-thread` + `macros` 等 feature。
- **`serde = { features = ["derive"] }`**：啟用 `#[derive(Deserialize)]`，讓 struct 自動具備從 JSON 反序列化的能力。詳見 ch00〈什麼是 `features = ["derive"]`？〉小節。
- **`serde_json = "1"`**：純版本號、無 features。本章主要靠 reqwest 的 `json` feature 做反序列化，serde_json 是它的底層依賴（reqwest `json` feature 會拉進來），但顯式列出方便讀者理解。

### 安裝指令對照

```bash
# 方法一：cargo add（推薦）
cargo add reqwest --features json
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json

# 方法二：直接編輯 [dependencies] 區塊（如上面的 Cargo.toml 所示）
```

Python 對照：相當於 `pip install requests`（async 版還要 `httpx` + `asyncio`），但 Rust 把職責拆得更細。注意本章需要網路連線才能跑（會打 `https://httpbin.org`）。

## Python 對照

| 概念 | Python（`requests`） | Rust（`reqwest` + `serde`） |
|---|---|---|
| HTTP client | `requests` 套件 | `reqwest` crate |
| 建立 client | （通常直接用模組函式） | `reqwest::Client::new()` |
| GET 請求 | `requests.get(url)` | `client.get(url).send().await?` |
| POST + JSON | `requests.post(url, json=data)` | `client.post(url).json(&data).send().await?` |
| 剖析 JSON 回應 | `resp.json()`（回傳 `dict`） | `resp.json::<T>().await?`（需指定型別 `T`） |
| 錯誤狀態碼 | `resp.raise_for_status()` | `resp.error_for_status()?` |
| JSON 結構 | 執行期才知道的 `dict` | 編譯期就確定的 `struct` |

並排對照一個完整的「GET + 剖析 JSON」流程：

```python
# Python
import requests

resp = requests.get("https://jsonplaceholder.typicode.com/todos/1")
resp.raise_for_status()
todo = resp.json()          # dict，結構要到執行期才知道
print(todo["title"])        # 打錯 key 不會在編譯期被抓到
```

```rust
// Rust
#[derive(serde::Deserialize, Debug)]
struct Todo {
    #[serde(rename = "userId")]
    user_id: u32,
    id: u32,
    title: String,
    completed: bool,
}

let client = reqwest::Client::new();
let resp = client
    .get("https://jsonplaceholder.typicode.com/todos/1")
    .send()
    .await?
    .error_for_status()?;
let todo: Todo = resp.json().await?;   // 強型別，結構編譯期就確定
println!("{}", todo.title);            // 欄位打錯會編譯失敗
```

## 概念講解

### reqwest 的非同步設計

`reqwest::Client` 是**非同步**的，它底層建構在 `hyper` 與 `tokio` 之上，因此
**必須在 Tokio runtime 內使用**。這就是為什麼本章的 `main` 要加上
`#[tokio::main]` 並宣告為 `async fn`。

> 補充：reqwest 也提供同步的 `reqwest::blocking` API，但它是一個**預設關閉的
> feature**，必須在 `Cargo.toml` 額外啟用 `features = ["blocking"]` 才能用。
> 本教程選擇用非同步版本，因為它與第 3 章（async）和第 6 章（axum web）一脈相承。

### 建立可重用的 Client

```rust
let client = reqwest::Client::new();
```

`Client` 內部會維護一個 **keep-alive 連線池（connection pool）**。對同一個
host 發送多次請求時，可以重用已建立的 TCP／TLS 連線，省下重新交握的成本。

reqwest 也提供一個捷徑 `reqwest::get(url)`，但它**每次呼叫都會建立一個新的
`Client`**，無法重用連線池。官方文件明確建議：**若要發送多個請求，就建立一個
`Client` 並重用它**。本章採用這個推薦做法。

### 用 serde 把 JSON 對應到 struct

serde 透過 `#[derive(Deserialize)]` 自動產生「JSON → struct」的轉換程式碼。
對應規則預設是**欄位名稱完全相同**：

```rust
#[derive(Deserialize, Debug)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
}
```

但 JSONPlaceholder 的 Todo 有一個 camelCase 欄位 `userId`，而 Rust 慣用
snake_case（`user_id`）。這時用 `#[serde(rename = "...")]` 告訴 serde 這個
Rust 欄位在 JSON 裡的實際名稱：

```rust
#[derive(Deserialize, Debug)]
struct Todo {
    #[serde(rename = "userId")]
    user_id: u32,   // JSON 裡是 "userId"
    id: u32,
    title: String,
    completed: bool,
}
```

這是處理「JSON 命名慣例 ≠ Rust 命名慣例」最常見的手法。

### `error_for_status()` 把錯誤狀態碼轉成 `Err`

reqwest 的 `.send()` **不會**因為 4xx／5xx 回應而回傳 `Err`——它只會在
**網路層錯誤**（連不上、逾時、TLS 失敗等）時才 `Err`。拿到一個 404 回應時，
`.send()` 仍會成功回傳 `Response`。

若你希望「HTTP 狀態碼非 2xx 就視為錯誤」，要在拿到 `Response` 後呼叫
`.error_for_status()?`，它會把 4xx／5xx 轉成 `Err`，2xx 則回傳原本的
`Response`。這對應 Python 的 `resp.raise_for_status()`。

### `resp.json::<T>()`：編譯期就知道結構

```rust
let todo: Todo = resp.json().await?;
```

`json()` 是一個泛型方法，回傳型別由呼叫端的型別標注決定。這裡我們標注
`let todo: Todo`，所以 serde 會把 body 當 JSON 剖析成 `Todo`。

**Python 開發者注意**：Python 的 `resp.json()` 回傳一個 `dict`，結構要到執行期
才知道；Rust 的 `resp.json::<Todo>().await?` 需在呼叫處指定型別 `T`，因此
**結構在編譯期就確定**。欄位名打錯、型別不符，都會在編譯階段直接失敗，而非
等到執行時才炸開。

此外，serde 預設會**忽略 JSON 中未在 struct 定義的欄位**，不像 Pydantic
預設會對多餘欄位報錯。若想在多餘欄位出現時報錯，可加上
`#[serde(deny_unknown_fields)]`。

## 程式碼解析

以下逐段說明 `src/main.rs`。

### 1. 定義資料結構

```rust
#[derive(Deserialize, Debug)]
struct Todo {
    #[serde(rename = "userId")]
    user_id: u32,
    id: u32,
    title: String,
    completed: bool,
}
```

- `Deserialize`：讓 serde 能把 JSON 轉成 `Todo`。
- `Debug`：讓我們可以用 `{:#?}` 美化印出整個 struct。
- `#[serde(rename = "userId")]`：把 Rust 的 `user_id` 對應到 JSON 的
  `userId`，解決 camelCase／snake_case 不一致的問題。

### 2. 非同步 main 與可重用的 Client

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    // ...
}
```

- `#[tokio::main]` 把 `async fn main` 展開成「建立 Tokio runtime → 執行
  async main」的同步 `main`。
- 回傳 `Result<(), Box<dyn std::error::Error>>` 讓我們能用 `?` 傳播任何實作
  `std::error::Error` 的錯誤（reqwest 與 serde 的錯誤都符合），最後以 `Ok(())`
  結束。

### 3. GET 請求並剖析

```rust
let resp = client
    .get("https://jsonplaceholder.typicode.com/todos/1")
    .send()
    .await?
    .error_for_status()?;

let todo: Todo = resp.json().await?;

println!("GET /todos/1:\n{todo:#?}");
```

- `client.get(url)` 建立一個 GET `RequestBuilder`，`.send().await?` 送出。
- `.error_for_status()?` 把 4xx／5xx 轉成 `Err`。
- `resp.json().await?` 剖析 JSON；型別由 `let todo: Todo` 標注決定。
- `{todo:#?}` 用美化 Debug 格式印出。

### 4. POST 請求與動態 JSON

```rust
let new_todo = serde_json::json!({
    "title": "學 Rust",
    "completed": false,
    "userId": 1,
});

let resp = client
    .post("https://jsonplaceholder.typicode.com/todos")
    .json(&new_todo)
    .send()
    .await?
    .error_for_status()?;

let body: serde_json::Value = resp.json().await?;
println!("POST /todos (echo):\n{body:#?}");
```

- `serde_json::json!({ ... })` 巨集建立一個動態的 `serde_json::Value`，類似
  Python 的 `dict`。適合「結構不固定、或只是一次性送出」的場合。
- `client.post(url).json(&value)` 會自動把 `value` 序列化成 JSON 放進 body，
  並設定 `Content-Type: application/json`。
- 這次用 `serde_json::Value` 接住回應（任意 JSON），展示「動態 JSON」的寫法。
  JSONPlaceholder 的 POST 會回傳我們送出的內容並附上一個假 id。

## 執行方式

> **注意：本章需要網路連線**，程式會連到 `https://jsonplaceholder.typicode.com`
> 抓取與送出資料。請確認網路通暢。

```bash
# 從 workspace 根目錄執行
cargo run -p ch04-networking
```

預期輸出（節錄）：

```text
GET /todos/1:
Todo {
    user_id: 1,
    id: 1,
    title: "delectus aut autem",
    completed: false,
}
POST /todos (echo):
Object {
    "title": String("學 Rust"),
    "completed": Bool(false),
    "userId": Number(1),
    "id": Number(201),
}
```

- `GET /todos/1` 印出剖析後的 `Todo` struct（強型別，欄位名稱是 Rust 的
  snake_case）。
- `POST /todos (echo)` 印出伺服器 echo 回來的 JSON，以 `serde_json::Value` 的
  Debug 格式呈現，可看到多了一個假的 `id`。

## 重點回顧

- **reqwest 是非同步的**：預設的 `reqwest::Client` 需要 Tokio runtime，`main`
  要加 `#[tokio::main]`。同步的 `blocking` API 是預設關閉的 feature。
- **建立一個 `Client` 並重用它**：它內部維護 keep-alive 連線池，比每次呼叫
  `reqwest::get()` 更有效率。
- **serde 強型別 JSON**：`#[derive(Deserialize)]` 把 JSON 對應到 struct，
  欄位結構在編譯期就確定；用 `#[serde(rename = "...")]` 處理命名不一致。
- **`error_for_status()`**：把 4xx／5xx 回應轉成 `Err`，對應 Python 的
  `raise_for_status()`。
- **`resp.json::<T>()` vs Python `resp.json()`**：Rust 需指定型別 `T`（編譯期
  確定結構），Python 回傳動態 `dict`。serde 預設忽略未知欄位。

## 練習（選做）

1. 抓取**整份待辦清單**（多筆），用 `Vec<Todo>` 反序列化並印出筆數與前幾筆：
   ```rust
   let resp = client
       .get("https://jsonplaceholder.typicode.com/todos")
       .send()
       .await?
       .error_for_status()?;
   let todos: Vec<Todo> = resp.json().await?;
   println!("共 {} 筆", todos.len());
   for t in todos.iter().take(5) {
       println!("- {}", t.title);
   }
   ```
2. 練習用 `#[serde(deny_unknown_fields)]`，觀察當 JSON 含未定義欄位時會在剖析
   時報錯（與 Pydantic 預設行為一致）。
3. 把 GET 包成函式 `async fn fetch_todo(client: &Client, id: u32) -> Result<Todo, Box<dyn std::error::Error>>`，體驗把 `&Client` 傳進去重用的寫法。
