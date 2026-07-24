# 第 7 章：Rust 巨集（`#[...]` 與 `macro_rules!`）

讀到這裡，你已經在前六章**用過**很多次 `#[...]` 了：`#[derive(Debug/Serialize/Clone/Parser)]`、`#[allow(dead_code)]`、`#[serde(rename=...)]`、`#[tokio::main]`、`#[command(...)]`、`#[arg(...)]`。但你可能從沒想過：這些 `#[...]` 到底是什麼？為什麼能「一行指令就自動長出方法」？本章就把這層黑箱打開，並教你用 `macro_rules!` **自己寫巨集**。

## 學習目標

讀完本章後，你應該能：

- 理解 Rust 巨集的本質：**編譯期程式碼生成**，與 Python decorator（執行期）的根本差異。
- 分辨「屬性巨集 `#[...]`」「derive 巨集 `#[derive(...)]`」「宣告巨集 `macro_rules!`」三類。
- 學會用 `macro_rules!` 自訂巨集：片段型別（`$x:expr`）、重複匹配（`$($x),*`）、條件分支。
- 認識 `#[test]` 屬性巨集與 `cargo test`（補上教程從未示範的測試慣例）。

## 本章相依套件與 Cargo.toml

本章是教程中**唯一無外部相依**的應用章節。理由很簡單：**巨集是語言內建**。`macro_rules!`、`#[derive]`、`#[test]` 都來自編譯器與標準函式庫，不需要 `cargo add` 任何東西。完整 `Cargo.toml`：

```toml
[package]
name = "ch07-macros"
version = "0.1.0"
edition = "2021"

[dependencies]

[[bin]]
name = "ch07-macros"
path = "src/main.rs"
```

### 各「套件」用途說明

本章**無外部相依**--巨集是語言內建：

| 來源 | 巨集 / 屬性 | 說明 |
|---|---|---|
| 編譯器內建 | `macro_rules!` | 宣告巨集，本章核心，自己寫 |
| 標準函式庫 | `#[derive(Debug/Clone/PartialEq/...)]` | derive 巨集，前面各章用過 |
| 標準函式庫 | `#[test]`、`#[cfg(test)]` | 屬性巨集 / 條件編譯屬性 |
| 標準函式庫 | `println!`、`vec!`、`assert_eq!` | 宣告巨集，每天都在用 |

Python 對照：Python 要用 decorator 需 `import functools`、要做「程式碼生成」需 `exec()` 或第三方（如 `sympy`、`ast`）；Rust 巨集零相依，因為巨集是語言的一部分。

安裝指令：**不需 `cargo add` 任何東西**，`cargo new` 即可。

## Python 對照

| 概念 | Python | Rust |
|---|---|---|
| 編譯期程式碼生成 | 無（`exec()` / 字串拼接，危險） | 巨集（`macro_rules!` / derive / 屬性） |
| 裝飾器 | `@decorator`（執行期） | `#[attribute]`（編譯期，性質不同） |
| 資料類別自動生成 | `@dataclass` 產生 `__init__`/`__repr__` | `#[derive(Debug, Clone, PartialEq)]` 產生 trait impl |
| 可變參數 | `*args`（執行期 tuple） | `macro_rules!` 的 `$($x:expr),*`（編譯期展開） |
| 測試 | `pytest` 函數 | `#[test]` 屬性巨集 + `cargo test` |

並排版碼對照（derive 巨集 vs `@dataclass`）：

```python
# Python
from dataclasses import dataclass

@dataclass
class Point:
    x: int
    y: int
# dataclass 自動產生 __init__ / __repr__ / __eq__
```

```rust
// Rust
#[derive(Debug, Clone, PartialEq)]
struct Point { x: i32, y: i32 }
// derive 巨集自動產生 impl Debug / Clone / PartialEq for Point
```

兩者都是「在類別/struct 上加一個標記，就自動長出方法」，但 `@dataclass` 是 Python 在**執行期**用裝飾器改寫類別，`#[derive(...)]` 是 Rust 編譯器在**編譯期**把樣板程式碼展開出來。後者沒有執行期成本、型別更嚴格。

## 概念講解

### 1. 巨集是什麼？為什麼 Rust 需要它

一句話：**巨集 = 「寫程式碼的程式碼」，在編譯期跑**。

你寫一段巨集，編譯器會在**編譯期**把它展開成普通的 Rust 程式碼，然後才對展開後的程式碼做型別檢查與編譯。這與函數完全不同：函數是在**執行期**被呼叫的。

為什麼需要巨集？因為有些事函數做不到：

- 函數的型別在編譯期就固定，但巨集可以「每次呼叫產生型別特定的程式碼」（例如 `vec![1, 2, 3]` 與 `vec!["a", "b"]` 是不同型別的 Vec）。
- 函數不能改變語法結構，但巨集可以產生 `impl` 區塊、新 struct、新 `fn`--例如 `#[derive(Debug)]` 會產生一整個 `impl Debug for Point` 區塊。

對照 Python：Python 沒有編譯期巨集。最接近的概念是 **decorator**（`@something`），但 decorator 是在**執行期**執行的函數，接收一個函數/類別、回傳新的；Rust 的 `#[...]` 則是**編譯期**的程式碼變換。兩者語法相似、本質不同。另一種「程式碼生成」的做法是 Python 的 `exec()` 或字串拼接，但那完全失去型別檢查、危險且難除錯--Rust 巨集則是編譯期展開、展開後照樣過型別檢查。

### 2. 三種巨集總覽

Rust 的巨集分三大類：

| 類型 | 語法 | 誰寫的 | 本章是否教開發 |
|---|---|---|---|
| **宣告巨集** | `macro_rules! name { ... }` | 你自己 | ✅ 教（核心） |
| **derive 巨集** | `#[derive(Trait)]` | crate 作者（用 proc-macro） | ❌ 只教使用 |
| **屬性巨集** | `#[test]`、`#[tokio::main]`、`#[serde(...)]` | crate 作者（用 proc-macro）或編譯器內建 | ❌ 只教使用 |

為什麼不教 derive 巨集與屬性巨集的「開發」？因為那需要：

1. 一個獨立的 `proc-macro` crate（不能與應用程式同一個 crate）。
2. 用 `syn` 解析 Rust 語法樹、用 `quote` 產生程式碼。
3. 理解 TokenStream、解析器、展開器等編譯原理概念。

這對 Python 開發者負擔過重，也偏離入門定位。本章聚焦 `macro_rules!`--它已足夠建立「巨集 = 編譯期程式碼生成」的心智模型，且能實際動手跑。想學 derive 巨集開發，可參考 [The Book 第 19 章](https://doc.rust-lang.org/book/ch19-06-macros.html) 與 [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop)。

### 3. 你已經用過的 derive 巨集

回顧前面各章用過的 derive：

| derive | 出現章節 | 它做了什麼 |
|---|---|---|
| `#[derive(Debug)]` | ch00 | 產生 `impl Debug`，讓 `{:?}` 與 `dbg!` 能用 |
| `#[derive(Serialize, Deserialize)]` | ch01 / ch04 / ch06 | 產生 serde 的序列化/反序列化 impl |
| `#[derive(Clone)]` | ch00 / ch06 | 產生 `clone()` 方法 |
| `#[derive(Parser)]` | ch01 | clap 產生命令列解析邏輯 |

關鍵理解：`#[derive(Debug)]` 不是「編譯器幫你印東西」，而是編譯器在編譯期**把這行展開成一整個 impl 區塊**，例如：

```rust
#[derive(Debug)]
struct Point { x: i32, y: i32 }
```

展開後等價於（概念上）：

```rust
struct Point { x: i32, y: i32 }

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
```

所以你之後寫 `println!("{:?}", point)` 才能用--因為 `Point` 真的有了 `Debug` trait 的實作。對照 Python 的 `@dataclass` 產生 `__init__`/`__repr__`，是同樣的「加標記就長方法」概念，但 derive 在編譯期展開、無執行期成本。

### 4. 屬性巨集 `#[...]`

屬性巨集是「附加在 item（struct/fn/mod）上的編譯器指令」。前面各章用過的：

| 屬性 | 出現章節 | 性質 |
|---|---|---|
| `#[allow(dead_code)]` | ch00 | 內建屬性（調整 lint，不產生程式碼） |
| `#[serde(rename = "is_done")]` | ch04 | 屬性巨集（影響 serde 展開行為） |
| `#[tokio::main]` | ch03 / ch04 / ch06 | 屬性巨集（把 `fn main` 展開成 async runtime 入口） |
| `#[command(...)]`、`#[arg(...)]` | ch01 | 屬性巨集（clap 的設定） |

這裡要分清楚兩種東西：

- **內建屬性**（如 `#[allow]`、`#[cfg]`、`#[inline]`）：編譯器內建的指令，只是調整編譯行為（lint、條件編譯等），不會「產生新程式碼」。
- **屬性巨集**（如 `#[tokio::main]`、`#[serde(...)]`）：由 proc-macro crate 提供，會在編譯期**展開產生新程式碼**。例如 `#[tokio::main]` 會把：

```rust
#[tokio::main]
async fn main() { /* ... */ }
```

展開成（概念上）：

```rust
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { /* 原本的 async fn main 內容 */ })
}
```

本章不深入區分兩者，重點是建立「`#[...]` 是巨集家族」的直覺。你只需要知道：看到 `#[...]`，就是編譯期在對這個 item 做某種變換或設定。

### 5. `macro_rules!`：自己寫宣告巨集

這是本章核心。我們從最簡單開始，逐步加複雜度。

#### 最簡：無參數巨集

```rust
macro_rules! say_hello {
    () => {
        println!("hello from macro!");
    };
}
```

呼叫 `say_hello!()` 時，編譯器會把它**展開**成 `println!("hello from macro!");`。展開前 -> 展開後：

```text
say_hello!()          ->    println!("hello from macro!");
```

語法結構：

- `macro_rules! 名稱 { ... }`：定義一個宣告巨集。
- `() => { ... };`：一條「匹配規則（arm）」。`()` 是匹配模式（這裡是空，表示不接參數），`=>` 後是展開的程式碼。
- 每條 arm 用 `;` 結尾（最後一條可省略，但習慣上都寫）。

#### 帶參數：片段（fragment）

```rust
macro_rules! greet {
    ($name:expr) => {
        println!("你好, {}!", $name);
    };
}
```

- `$name` 是一個 **metavariable（元變數）**，名字自取。
- `:expr` 是 **片段型別（fragment specifier）**，表示「匹配一個表達式（expression）」。
- 展開時，`$name` 會被替換成你傳入的那個表達式。

```text
greet!("小明")    ->    println!("你好, {}!", "小明");
greet!(1 + 2)     ->    println!("你好, {}!", 1 + 2);   // 印出 "你好, 3!"
```

注意 `greet!("小明")` 與 `greet!(1 + 2)` 用的是**同一個巨集**，但展開出的程式碼型別不同（一個是 `&str`、一個是 `i32`）。這就是巨集與函數的關鍵差別：函數要靠泛型才能處理多型別，巨集是「每次呼叫產生型別特定的程式碼」，不需泛型。

常見片段型別：

| 片段 | 匹配什麼 | 範例 |
|---|---|---|
| `expr` | 表達式 | `1 + 2`、`foo()`、`"hi"` |
| `ident` | 識別字 | `x`、`Point`、`my_func` |
| `ty` | 型別 | `i32`、`Vec<String>`、`&'a str` |
| `tt` | 單一 token tree | 任何一個括號內的東西 |
| `literal` | 字面值 | `42`、`"hi"`、`true` |
| `block` | 程式區塊 | `{ let x = 1; x }` |
| `pat` | pattern | `Some(x)`、`_`、`1..=10` |

#### 重複匹配：`$($x),*`

```rust
macro_rules! my_vec {
    ($($x:expr),*) => {
        {
            let mut v = Vec::new();
            $( v.push($x); )*
            v
        }
    };
}
```

這是 `vec!` 巨集的核心手法。拆解語法：

- `$($x:expr),*`：匹配「零或多個 expr，用逗號分隔」。
  - `$(` ... `)` 包住要重複的部分（這裡是 `$x:expr`）。
  - `,` 是分隔符。
  - `*` 表示「重複零次以上」（`+` 則是「一次以上」）。
- `$( v.push($x); )*`：在展開端**同樣的 `$(...)*` 語法**，會把每個匹配到的 `$x` 展開成 `v.push($x);`。

展開過程圖解：

```text
my_vec!(1, 2, 3)
  ->  {
          let mut v = Vec::new();
          v.push(1); v.push(2); v.push(3);   // $( v.push($x); )* 展開三次
          v
      }

my_vec!()          // 零個參數：$($x),* 匹配空，$( ... )* 展開成空
  ->  {
          let mut v = Vec::new();
          v
      }
```

對照 Python：無直接對應，最接近的是 `*args` 展開，但 `*args` 是執行期收集成 tuple，巨集則是在編譯期「長出 N 個 `push` 呼叫」。

#### 條件分支：多 arm

```rust
macro_rules! count {
    () => { 0usize };
    ($x:expr) => { 1usize };
    ($x:expr, $y:expr) => { 2usize };
}
```

巨集可有多條 arm，編譯器會**依序嘗試**，用第一個能匹配的。這像 `match`，但發生在編譯期。

```text
count!()        ->  0usize     // 匹配第一條 arm
count!(9)       ->  1usize     // 匹配第二條
count!(1, 2)    ->  2usize     // 匹配第三條
```

> 注意：這個 `count!` 只處理 0/1/2 個參數。要處理任意數量需用更進階的「計數巨集」技巧（用 `$()` 重複 + 遞迴），超出本章範圍。

### 6. 巨集 vs 函數

| 面向 | 巨集 | 函數 |
|---|---|---|
| 何時執行 | 編譯期展開 | 執行期呼叫 |
| 型別 | 每次展開產生型別特定程式碼 | 固定型別（除非用泛型） |
| 可否產生新語法 | ✅ 可產生 `impl`、`struct`、`fn` | ❌ 不能 |
| 可變參數 | ✅ 用 `$($x),*` | ❌ 需泛型或 tuple trick |
| 除錯 | 較難（展開後程式碼不可見） | 容易（有明確呼叫棧） |
| 效能 | 編譯期已展開，無執行期成本 | 有呼叫成本（常被 inline 消除） |

關鍵例子：`greet!(1 + 2)` 與 `greet!("小明")` 不需泛型，因為巨集是「文字替換式展開」，每次展開產生型別特定的程式碼。如果要用函數做到，得寫泛型 `fn greet<T: Display>(name: T)`，而且還受 `Display` trait 限制。

實務原則：**能用函數就用函數**，巨集留給「函數做不到的事」（產生 struct/impl、可變參數、DSL）。標準函式庫的 `vec!`、`println!`、`format!` 都是巨集，正因為它們需要可變參數。

### 7. `#[test]`：屬性巨集的實戰

`#[test]` 是屬性巨集的天然範例，且教程從未示範測試，本章順帶補上。

慣例寫法：把測試函數放在 `#[cfg(test)] mod tests` 裡，每個測試函數標 `#[test]`：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_clone_is_equal() {
        let p = Point { x: 5, y: 10 };
        assert_eq!(p, p.clone());
    }
}
```

重點：

- `#[cfg(test)]`：條件編譯屬性，告訴編譯器「這個 mod 只在 `cargo test` 時編譯」，平常 `cargo build`/`cargo run` 不會編進執行檔。
- `#[test]`：屬性巨集，把普通函數標記成「測試 harness 可識別的測試案例」。`cargo test` 會自動執行所有標 `#[test]` 的函數。
- `assert_eq!(a, b)`：斷言兩值相等（這本身也是巨集！失敗時會印出兩邊的值）。還有 `assert!(cond)` 斷言為真、`assert_ne!(a, b)` 斷言不相等。
- `use super::*`：把上層模組（`main.rs` 的內容）的東西引入測試模組。

執行 `cargo test` 的流程：

1. 編譯器編譯主程式 + 測試模組（因為 `cfg(test)` 在測試建置時為真）。
2. Rust 內建測試 harness 收集所有 `#[test]` 函數。
3. 逐一執行，印出 `test tests::xxx ... ok` 或 `... FAILED`。

對照 Python：`#[test]` 之於 `cargo test`，就像 `def test_xxx():` 之於 `pytest`。差別是 Rust 的測試是編譯器 + 語言層級整合，不需安裝額外框架。

## 程式碼解析

本章 `src/main.rs` 分三段。

### 第 1 段：derive 巨集示範

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point { x: i32, y: i32 }
```

一行 derive 就讓 `Point` 擁有三個 trait 的實作。`main` 裡示範它們產生的能力：

```rust
let p1 = Point { x: 1, y: 2 };
let p2 = p1.clone();              // Clone 產生的 clone()
println!("{:?}", p1);             // Debug 產生的 {:?} 支援
println!("p1 == p2? {}", p1 == p2);  // PartialEq 產生的 ==
```

對照 Python：相當於 `@dataclass` 一次產生 `__init__`/`__repr__`/`__eq__`。

### 第 2 段：四個 `macro_rules!` 巨集

#### `say_hello!`（無參數）

```text
say_hello!()  ->  println!("hello from macro!");
```

#### `greet!`（帶一個 expr）

```text
greet!("小明")  ->  println!("你好, {}!", "小明");
greet!(1 + 2)   ->  println!("你好, {}!", 1 + 2);   // 印 "你好, 3!"
```

同一個巨集、不同型別參數--這是函數做不到的（除非泛型）。

#### `my_vec!`（重複匹配）

```text
my_vec!(1, 2, 3, 4, 5)
  ->  { let mut v = Vec::new(); v.push(1); v.push(2); v.push(3); v.push(4); v.push(5); v }
```

`$($x:expr),*` 匹配五個 expr，`$( v.push($x); )*` 展開成五個 push 呼叫。

#### `count!`（條件分支）

三條 arm 依參數數量匹配，編譯期決定展開成 `0usize`/`1usize`/`2usize`。

### 第 3 段：`#[test]` + `mod tests`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_clone_is_equal() { /* ... */ }

    #[test]
    fn my_vec_builds_correctly() { /* ... */ }

    #[test]
    fn count_macro_branches() { /* ... */ }
}
```

三個測試分別驗證：derive 的 Clone/PartialEq、`my_vec!` 的重複匹配（含空呼叫）、`count!` 的三條分支。注意空 `my_vec!()` 的測試需用型別標注 `let empty: Vec<i32> = my_vec!();`，因為空 Vec 無法從參數推導出元素型別--這是 Rust 型別推導的限制，與巨集無關。

## 執行方式

從 workspace 根目錄：

```bash
# 執行主程式：印出 derive 示範 + 4 個巨集示範輸出
cargo run -p ch07-macros
```

預期輸出：

```text
Point { x: 1, y: 2 }
p1 == p2? true
hello from macro!
你好, 小明!
你好, 3!
my_vec! => [1, 2, 3, 4, 5]
count!() = 0
count!(1) = 1
count!(1, 2) = 2
```

```bash
# 跑測試（本章獨有，教程首個示範 cargo test）
cargo test -p ch07-macros
```

預期輸出：

```text
running 3 tests
test tests::point_clone_is_equal ... ok
test tests::my_vec_builds_correctly ... ok
test tests::count_macro_branches ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

也可以從單章目錄執行：

```bash
cd chapters/ch07-macros
cargo run
cargo test
```

## 重點回顧

1. **巨集 = 編譯期程式碼生成**，與 Python decorator（執行期）本質不同。Python 沒有編譯期巨集，最接近的是 `exec()`（危險、無型別檢查）。
2. **三類巨集**：`macro_rules!`（宣告式，本章教寫）、derive 巨集（`#[derive(T)]`，各章用過）、屬性巨集（`#[test]`/`#[tokio::main]`）。後兩者的「開發」需 proc-macro crate + syn/quote，超出本章範圍。
3. **`macro_rules!` 核心**：`$x:fragment` 片段（`expr`/`ident`/`ty` 等）、`$($x),*` 重複匹配、多 arm 條件分支。
4. **巨集 vs 函數**：編譯期展開、可接受任意型別、可產生新語法，但除錯較難。實務原則：能用函數就用函數。
5. **`#[test]` + `#[cfg(test)] mod tests`** 是 Rust 測試慣例，`cargo test` 執行。`assert_eq!`/`assert!` 本身也是巨集。

## 練習

### 練習 1：`max_of!` 巨集（難度 ⭐⭐）

寫一個 `macro_rules! max_of!` 接受 2 個 expr，回傳較大值。

提示：

```rust
macro_rules! max_of {
    ($a:expr, $b:expr) => {
        // 想想：用 if 表達式，$a > $b 時回 $a，否則回 $b
    };
}
```

參考答案（先自己試再看）：

```rust
macro_rules! max_of {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    };
}

// 用法：assert_eq!(max_of!(3, 7), 7);
```

### 練習 2：`hashmap!` 巨集（難度 ⭐⭐）

寫一個 `macro_rules! hashmap!` 接受 `key => value, ...` 語法，產生 `HashMap`。

提示：

- 重複匹配用 `$($k:expr => $v:expr),*`（注意分隔符是 `=>` 在 expr 之間）。
- 需 `use std::collections::HashMap;`。

```rust
macro_rules! hashmap {
    ($($k:expr => $v:expr),*) => {
        {
            let mut m = HashMap::new();
            // 想想：怎麼用 $( ... )* 把每組 k/v 插進去？
            m
        }
    };
}
```

參考答案（先自己試再看）：

```rust
use std::collections::HashMap;

macro_rules! hashmap {
    ($($k:expr => $v:expr),*) => {
        {
            let mut m = HashMap::new();
            $( m.insert($k, $v); )*
            m
        }
    };
}

// 用法：
// let m = hashmap!("a" => 1, "b" => 2);
// assert_eq!(m.get("a"), Some(&1));
```

這個練習綜合了重複匹配（`$($k:expr => $v:expr),*`）與展開（`$( m.insert($k, $v); )*`），是 `my_vec!` 的進階版--差別在每個重複單位有兩個 metavariable（key 與 value）。
