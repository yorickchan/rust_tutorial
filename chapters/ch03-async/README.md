# 第 3 章：非同步程式設計（tokio）

## 學習目標

- 理解 Rust 的 `async`/`await` 與 `Future` 的運作方式。
- 學會用 `tokio` 建立非同步 runtime 並執行並發任務。
- 比較「循序（sequential）」與「並發（concurrent）」執行的耗時差異。
- 認識 `tokio::spawn` 對 `Send` 的要求，以及它與 Python `asyncio` 的關鍵差別。

## Python 對照

對 Python 開發者來說，Rust 的 `async`/`await` 在語法上與 `asyncio` 非常相似，但背後的執行模型與型別限制不同。

### 概念對應表

| Python | Rust |
|---|---|
| `asyncio`（標準庫事件迴圈） | `tokio`（非同步 runtime crate） |
| `async def task()` | `async fn task()` |
| `await asyncio.sleep(1)` | `tokio::time::sleep(...).await` |
| `asyncio.create_task(coro)` | `tokio::spawn(future)` |
| `asyncio.gather(a, b)` | 多個 `tokio::spawn` 後逐一 `.await` |
| `loop.run_until_complete(main())` | `#[tokio::main] async fn main()` |

### 並排版碼對照

定義一個非同步任務：

```python
# Python
async def task(name, ms):
    print(f"開始 {name}")
    await asyncio.sleep(ms / 1000)
    print(f"完成 {name}")
```

```rust
// Rust
async fn task(name: &str, ms: u64) {
    println!("開始 {name}");
    tokio::time::sleep(Duration::from_millis(ms)).await;
    println!("完成 {name}");
}
```

並發執行兩個任務：

```python
# Python
await asyncio.gather(task("C", 1000), task("D", 1000))
```

```rust
// Rust
let h1 = tokio::spawn(task("C", 1000));
let h2 = tokio::spawn(task("D", 1000));
h1.await.unwrap();
h2.await.unwrap();
```

## 概念講解

### 1. `async fn` 回傳的是 `Future`，不會立即執行

呼叫一個 `async fn` 不會馬上執行它的內容，而是回傳一個「尚未執行的 `Future`」。這個 `Future` 只有在被 `.await` 或交給 `tokio::spawn` 排程後，才會開始（並持續）執行。這與 Python 中呼叫 `async def` 函式回傳一個 coroutine object（需 `await` 才執行）的概念一致。

### 2. `#[tokio::main]` 展開 `async fn main`

Rust 的真正入口 `fn main` 必須是同步的。`#[tokio::main]` 巨集把我們寫的 `async fn main` 展開成：一個同步 `fn main`，在裡面建立 tokio runtime，再把我們的非同步邏輯放進去執行。對應 Python 裡常見的 `asyncio.run(main())`。

```rust
#[tokio::main]
async fn main() {
    // 這裡可以用 .await
}
```

### 3. `tokio::spawn` 與 `JoinHandle`

`tokio::spawn(future)` 把一個 future 排到 runtime 上執行，並立即回傳一個 `JoinHandle<T>`。這代表這個 task 開始與主流程「並發」進行。之後對 `JoinHandle` 呼叫 `.await` 即可等待它完成並取回結果（型別為 `Result<T, JoinError>`，`Err` 表示該 task 內發生 panic）。

對應 Python 的 `asyncio.create_task` 回傳一個 `Task` 物件，`await task` 可等待其完成。

### 4. `.await` 是「非阻塞」等待

當某個 future 在 `.await` 處暫停（例如 `tokio::time::sleep` 尚未到時間），runtime 不會讓執行緒跟著發呆，而是切換去驅動其他「就緒」的 future。這正是「並發」能在單一執行緒（或多執行緒）runtime 上實現的關鍵。

本例中兩個 `task("C")` 與 `task("D")` 各 sleep 1 秒，但同時進行，所以總耗時約 1 秒，而非 2 秒。

### 5. `Send` 要求（Python 開發者注意）

`tokio::spawn` 要求被 spawn 的 future 及其內部捕獲（capture）的值都必須是 `Send`。這是因為 tokio 的多執行緒 runtime 會把 task 在不同工作執行緒之間移動；若 future 持有非 `Send` 的資料，跨執行緒移動會導致未定義行為，編譯器會直接拒絕。

實務上的影響：

- 跨 task 共用的可變狀態要用 `Arc<Mutex<T>>`，**不能**用 `Rc`、`RefCell`（這些不是 `Send`／`Sync`）。
- spawn 捕獲的借用以 `'static` 為宜。本例 `task("C", 1000)` 捕獲的是字串字面常數 `&'static str`，是 `Send` 的，所以可安全 spawn。切勿 spawn 捕獲非 `'static` 借用（例如指向區域 `String` 的 `&str`）的 future。

Python 的 `asyncio` 沒有這個限制，因為它本質上是單一執行緒事件迴圈加上 GIL，task 不會跨執行緒移動。

## 程式碼解析

以下逐段解析 `src/main.rs`。

### 引入與 runtime 進入點

```rust
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
```

`Duration` 用來指定睡眠時間，`Instant` 用來量測耗時。`#[tokio::main]` 讓 `async fn main` 得以成為程式進入點（內部建立 runtime）。

### 定義非同步任務

```rust
async fn task(name: &str, ms: u64) {
    println!("開始 {name}");
    tokio::time::sleep(Duration::from_millis(ms)).await;
    println!("完成 {name}");
}
```

呼叫 `task(...)` 只是產生一個 Future，真正執行要等 `.await` 或 `spawn`。`tokio::time::sleep(...).await` 是非阻塞等待，對應 Python 的 `await asyncio.sleep(...)`。

### 循序執行（約 2 秒）

```rust
let start = Instant::now();
task("A", 1000).await;
task("B", 1000).await;
println!("循序耗時: {:?}", start.elapsed());
```

兩個任務逐一 `.await`：A 完全跑完後才開始 B，兩者各 1 秒，合計約 2 秒。`start.elapsed()` 以 `Debug` 格式印出耗時。

### 並發執行（約 1 秒）

```rust
let start = Instant::now();
let h1 = tokio::spawn(task("C", 1000));
let h2 = tokio::spawn(task("D", 1000));
h1.await.unwrap();
h2.await.unwrap();
println!("並發耗時: {:?}", start.elapsed());
```

兩個任務同時被排程到 runtime 上，在各自的 `sleep` 處暫停時，runtime 切換去驅動另一個。兩者各 1 秒但並行，故合計約 1 秒。`h1.await.unwrap()` 中的 `unwrap()` 是因為 `JoinHandle::await` 回傳 `Result`，當 task 內 panic 時為 `Err`；此處為入門範例直接 unwrap。

### 比較輸出

```rust
println!("比較: 循序 ~2s vs 並發 ~1s");
```

執行後可清楚看到循序約 2 秒、並發約 1 秒的差異。

## 執行方式

```bash
cargo run -p ch03-async
```

預期輸出（耗時數值會因機器而略有不同）：

```text
開始 A
完成 A
開始 B
完成 B
循序耗時: 2.00s
---
開始 C
開始 D
完成 C
完成 D
並發耗時: 1.00s
---
比較: 循序 ~2s vs 並發 ~1s
```

觀察重點：

- 循序段 A、B 依序「開始→完成」；並發段 C、D 幾乎同時「開始」，再幾乎同時「完成」。
- 並發段耗時約為循序段的一半。

## 重點回顧

- `async fn` 回傳 `Future`，必須 `.await` 或 `spawn` 才會執行。
- `#[tokio::main]` 把 `async fn main` 展開成同步入口並建立 tokio runtime。
- `tokio::spawn` 在 runtime 上排程 task，回傳 `JoinHandle<T>`，`.await` 取回結果。
- `.await` 是非阻塞等待，等待期間 runtime 可切換到其他就緒的 task，這是並發的來源。
- `Send` 要求：被 spawn 的 future 與其捕獲值必須 `Send`；跨 task 共用可變狀態用 `Arc<Mutex<T>>`，不可用 `Rc`/`RefCell`。Python `asyncio` 因單執行緒 + GIL 無此限制。

## 練習（選做）

請試著 spawn 5 個 task，每個各 `sleep` 500ms，確認「並發」總耗時約為 500ms，而非循序的 2.5s。

提示：

```rust
let start = Instant::now();
let mut handles = Vec::new();
for i in 0..5 {
    handles.push(tokio::spawn(task(&format!("task-{i}"), 500)));
}
for h in handles {
    h.await.unwrap();
}
println!("並發 5 tasks 耗時: {:?}", start.elapsed());
```

想一想：若把 `task` 的第一個參數改成借用一個區域 `String`（非 `'static`），編譯會出現什麼錯誤？這正是 `Send` 與 `'static` 限制在實務上會遇到的情境。
