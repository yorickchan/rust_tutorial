# 第 1 章：CLI 工具（clap）

## 學習目標

- 學會用 `clap` 的 **derive API** 建立命令列工具。
- 理解 Rust 的命令列參數解析模式：位置參數（positional）、旗標（flag）、選用參數（optional）。
- 用 `enum` + `#[derive(Subcommand)]` 定義子命令（subcommand）。
- 對照 Python `argparse`，建立「Python → Rust」的心智模型。

---

## Python 對照

Python 開發者通常用標準庫 `argparse` 做 CLI。clap 的 derive API 概念相近，但把「參數定義」直接寫成 struct / enum 欄位，由巨集自動產生解析邏輯，型別也更安全。

| 概念 | Python (`argparse`) | Rust (`clap` derive) |
|---|---|---|
| 解析器 | `ArgumentParser()` | `#[derive(Parser)] struct Cli` |
| 子命令 | `add_subparsers()` | `#[command(subcommand)] command: Commands` |
| 個別子命令 | `add_parser("add")` | `enum Commands` 的 variant |
| 位置參數 | `add_argument("task")` | `task: String`（裸欄位） |
| 旗標 `--name` | `add_argument("--name")` | `#[arg(long)] name: Option<String>` |
| 短旗標 `-n` | `add_argument("-n")` | `#[arg(short, long)] name: Option<String>` |
| 說明文字 | `help="..."` | `///` doc comment |
| 自動 `--help` | 內建 | 內建 |
| 自動 `--version` | 需 `action="version"` | `#[command(version)]` 內建 |

### 並排程式碼對照

**Python（argparse 寫一個等價的 Todo CLI）**：

```python
import argparse

def main():
    parser = argparse.ArgumentParser(description="簡單 Todo CLI")
    sub = parser.add_subparsers(dest="command", required=True)

    p_add = sub.add_parser("add", help="新增待辦事項")
    p_add.add_argument("task", help="待辦內容")

    sub.add_parser("list", help="列出所有待辦事項")

    p_done = sub.add_parser("done", help="標記完成")
    p_done.add_argument("id", type=int, help="待辦編號")

    args = parser.parse_args()
    # ...依 args.command 分派...
```

**Rust（clap derive，同樣的結構）**：

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "簡單 Todo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 新增待辦事項
    Add { task: String },
    /// 列出所有待辦事項
    List,
    /// 將指定編號的待辦標記為完成
    Done { id: u32 },
}
```

可以明顯看到：Python 得用 `add_parser` 逐一設定，Rust 則把整個命令結構宣告在一個 `enum` 裡，編譯器還會檢查每個 variant 的欄位型別。

---

## 概念講解

### 1. clap 的 derive API

clap 提供兩種 API：builder（用方法鏈逐步組裝）與 derive（用巨集從型別定義自動產生）。本章用 **derive API**，因為它最接近「宣告式」的寫法——你定義「CLI 長什麼樣子」，clap 負責解析。

核心步驟：

1. 在 struct 上加 `#[derive(Parser)]`，這個 struct 就是頂層 CLI。
2. 用 `#[command(...)]` 設定整體屬性（如 `version`、`about`）。
3. 欄位對應參數；子命令用 `#[command(subcommand)]` 標記的 enum 欄位。

```rust
#[derive(Parser)]
#[command(version, about = "簡單 Todo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

呼叫 `Cli::parse()` 時，clap 會讀取 `std::env::args()` 並自動解析。解析失敗（缺參數、型別錯誤）時，clap 會印出錯誤訊息與 `--help` 並讓程式離開——你完全不用手寫錯誤處理。

### 2. 三種參數形式

這是 Python 開發者最容易搞混的地方，務必記住：

| 寫法 | 是什麼 | 範例 |
|---|---|---|
| `task: String`（裸欄位） | **必填位置參數** | `cargo run -- add "買牛奶"` |
| `#[arg(long)] name: Option<String>` | **選用旗標** `--name` | `cargo run -- --name foo` |
| `#[arg(short, long)] name: Option<String>` | **短+長旗標** `-n` / `--name` | `cargo run -- -n foo` |

#### ⚠️ Python 開發者注意

在 Python `argparse` 裡，`add_argument("task")`（無 `--`）是位置參數，`add_argument("--task")`（有 `--`）是旗標。這個區分在 clap derive 裡**改由欄位寫法決定**：

- `task: String`（裸欄位，沒有 `#[arg(...)]`）→ **位置參數**，**不是** `--task`！
- 想要 `--task` 這種旗標，要寫 `#[arg(long)] task: Option<String>`。
- 想要必填旗標，把 `Option<String>` 改成 `String` 並加 `#[arg(long)]`。

換句話說：**有沒有 `--` 取決於你有沒有加 `#[arg(long)]` / `#[arg(short)]`，跟欄位名稱無關。** 裸欄位一律是位置參數。

型別也由 Rust 的型別決定：`String` 接受任意字串、`u32` 會自動把字串解析成非負整數（解析失敗 clap 會報錯）、`Option<T>` 代表選用、`bool` 配 `#[arg(short, long)]` 常當開關旗標。

### 3. 子命令用 enum 定義

子命令（subcommand）是 CLI 工具常見模式，例如 `git add`、`git commit`。在 clap derive 裡，子命令用一個 `enum` 表達，每個 variant 是一個子命令，variant 的欄位就是該子命令的參數：

```rust
#[derive(Subcommand)]
enum Commands {
    /// 新增待辦事項
    Add { task: String },
    /// 列出所有待辦事項
    List,
    /// 將指定編號的待辦標記為完成
    Done { id: u32 },
}
```

- `Add { task: String }`：`add` 子命令，帶一個必填位置參數 `task`。
- `List`：沒有參數的子命令。
- `Done { id: u32 }`：`done` 子命令，帶一個 `u32` 位置參數，clap 會自動把輸入字串解析成數字。

`///` 開頭的 doc comment 會自動變成該子命令在 `--help` 裡的說明文字。

### 4. 為什麼還列了 serde / serde_json？

本章的 `Cargo.toml` 列了 `serde` 與 `serde_json`，但這個最小 CLI 沒有大量使用它們。原因是：讓 `Cargo.toml` 反映「一個真實 CLI 專案常見的依賴組合」——多數 CLI 會讀寫設定檔（JSON）、或把資料序列化存檔。保留這兩個依賴，方便你在練習題（檔案持久化）直接用上，不必再改 `Cargo.toml`。實務上，如果確定用不到，當然也可以移除以縮短編譯時間。

---

## 程式碼解析

以下是 `src/main.rs` 的逐段說明。

### 引入與頂層 CLI

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "簡單 Todo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

- `use clap::{Parser, Subcommand};`：`Parser` 是給頂層 struct 用的 trait，`Subcommand` 是給子命令 enum 用的。derive 巨集靠這兩個 trait 展開。
- `#[command(version, about = "...")]`：`version` 讓 clap 自動支援 `--version`（版本來自 `Cargo.toml`），`about` 是程式的一句話描述，會出現在 `--help` 開頭。
- `#[command(subcommand)] command: Commands`：這個欄位告訴 clap「第一個位置參數是子命令，請分派到 `Commands` enum」。

### 子命令 enum

```rust
#[derive(Subcommand)]
enum Commands {
    /// 新增待辦事項
    Add { task: String },
    /// 列出所有待辦事項
    List,
    /// 將指定編號的待辦標記為完成
    Done { id: u32 },
}
```

三個 variant 各對應一個子命令。`Add` 與 `Done` 帶位置參數，`List` 不帶。`id: u32` 让 clap 自動做字串→整數的轉換與驗證。

### 資料結構

```rust
#[derive(Debug)]
struct Todo {
    text: String,
    done: bool,
}
```

一個待辦項目有內容 `text` 與完成狀態 `done`。`#[derive(Debug)]` 讓它可用 `{:?}` 印出，方便除錯。

### main：解析與分派

```rust
fn main() {
    let cli = Cli::parse();
    let mut todos: Vec<Todo> = Vec::new();

    match cli.command {
        Commands::Add { task } => { /* ... */ }
        Commands::List => { /* ... */ }
        Commands::Done { id } => { /* ... */ }
    }
}
```

- `Cli::parse()`：讀命令列參數並解析成 `Cli` 結構。
- `let mut todos: Vec<Todo> = Vec::new();`：待辦清單。**注意它是記憶體內的**，每次 `cargo run` 都是新行程，從空開始。
- `match cli.command`：對子命令 enum 做 pattern matching 分派——這是 Rust 處理分派的慣用法，編譯器還會檢查你有沒有漏掉任何 variant。

### Add：新增

```rust
Commands::Add { task } => {
    let id = todos.len() + 1;
    todos.push(Todo { text: task, done: false });
    println!("已新增: \"{}\" (編號 {})", todos[id - 1].text, id);
}
```

把 `task` 直接 move 進 `Todo`（不 clone），再從 `Vec` 讀回來印，避免額外配置。編號是 1 起算（`len() + 1`）。

### List：列出

```rust
Commands::List => {
    if todos.is_empty() {
        println!("（無待辦事項）");
    } else {
        for (i, todo) in todos.iter().enumerate() {
            let idx = i + 1;
            if todo.done {
                println!("{}. {} [完成]", idx, todo.text);
            } else {
                println!("{}. {}", idx, todo.text);
            }
        }
    }
}
```

`enumerate()` 同時給索引與元素，把 0 起算的 `i` 轉成 1 起算的 `idx`。完成的項目後面加上 `[完成]` 標記。

### Done：標記完成

```rust
Commands::Done { id } => {
    let index = id as usize;
    if index == 0 || index > todos.len() {
        println!("錯誤: 找不到編號 {}", id);
    } else {
        let todo = &mut todos[index - 1];
        todo.done = true;
        println!("已完成: {}", todo.text);
    }
}
```

- `id as usize`：把 `u32` 轉成索引型別 `usize`。
- 邊界檢查：`index == 0`（避免 1 起算的編號 0）或超出長度都印錯誤。
- `&mut todos[index - 1]`：取得可變借用來修改 `done`。

---

## 執行方式

從 workspace 根目錄執行（`-p` 指定 package）：

```bash
cargo run -p ch01-cli -- add "買牛奶"
cargo run -p ch01-cli -- list
cargo run -p ch01-cli -- done 1
cargo run -p ch01-cli -- --help
```

> `--` 之後的參數會原樣傳給程式（而不是被 `cargo` 自己吃掉），所以子命令與位置參數都要放在 `--` 後面。

### 預期輸出

**`cargo run -p ch01-cli -- add "買牛奶"`**：

```
已新增: "買牛奶" (編號 1)
```

**`cargo run -p ch01-cli -- --help`**（示意，實際排版由 clap 產生）：

```
簡單 Todo CLI

Usage: ch01-cli <COMMAND>

Commands:
  add   新增待辦事項
  list  列出所有待辦事項
  done  將指定編號的待辦標記為完成
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### ⚠️ 關於「記憶體內」狀態

這個 CLI 的待辦清單存在 `Vec<Todo>`，而 `Vec` 活在行程記憶體內。**每一次 `cargo run` 都是一個全新行程**，所以：

```bash
cargo run -p ch01-cli -- add "買牛奶"   # 印出「已新增」，但資料隨行程結束消失
cargo run -p ch01-cli -- list           # 印出「（無待辦事項）」，因為是新行程
```

`add` 之後再 `list` 會是空的，這是預期行為，不是 bug。要做持久化，得把資料寫進檔案或資料庫——這正是練習題的方向。對 Python 開發者來說，這就像每次 `python todo.py add ...` 都重新啟動一支腳本，沒有任何跨呼叫的狀態一樣。

---

## 重點回顧

1. **derive API**：`#[derive(Parser)]` 從 struct/enum 定義自動產生參數解析，不必手寫 builder。
2. **子命令用 enum**：`#[derive(Subcommand)] enum` 的每個 variant 是一個子命令，欄位是其參數；用 `#[command(subcommand)]` 欄位接到頂層 CLI。
3. **位置參數 vs 旗標**：裸欄位（`task: String`）是位置參數；`#[arg(long)]` / `#[arg(short)]` 才是旗標（`--task` / `-t`）。這是 Python 開發者最常踩的點。
4. **型別即驗證**：`u32` 欄位讓 clap 自動把字串解析成整數，解析失敗自動報錯；`Option<T>` 代表選用。
5. **自動 `--help` / `--version`**：clap 內建，不需手寫；`///` doc comment 自動成為說明文字。
6. **記憶體內狀態**：`Vec<Todo>` 隨行程結束消失，跨執行不保留——要持久化需檔案/資料庫。

---

## 練習（選做）

1. **新增 `Remove { id: u32 }` 子命令**：依編號刪除待辦。提示：在 `Commands` enum 加一個 variant，在 `match` 加一個分支，用 `todos.remove(index - 1)` 刪除。
2. **檔案持久化**：把 `todos` 在程式結束前用 `serde_json` 序列化寫入 `todos.json`，啟動時讀回。這會讓 `add` 與 `list` 跨執行保留狀態，也是 `Cargo.toml` 裡 `serde`/`serde_json` 真正派上用場的地方。
3. **加上 `--priority` 旗標**：給 `Add` 一個選用的 `#[arg(long)] priority: Option<u32>`，讓使用者可指定優先順序。
