# 第 0 章：Rust 基礎（給 Python 開發者）

這一章是整個教程的起點。如果你已經會寫 Python，那麼 Rust 的語法對你來說不會太陌生；但 Rust 有幾個核心觀念是 Python 完全沒有的——尤其是 **ownership（所有權）**。本章會用 Python 對照的方式，帶你一次理解 Rust 的 7 個基礎概念。

## 學習目標

讀完本章後，你應該能：

- 理解 Rust 與 Python 的**核心差異**：所有權（ownership）、型別系統（type system）、錯誤處理（error handling）。
- 知道為什麼 Rust 變數**預設不可變**，以及如何用 `mut` 改變它。
- 看懂 ownership 轉移（move）、借用（borrow）、可變借用（mutable borrow）三者的差別與規則。
- 用 `Result` + `match` 取代 Python 的 `try/except`。
- 用 `struct`、`enum`、`match` 組織資料與邏輯。

## Rust 工具鏈簡介

在進入語法之前，先建立 Rust 工具鏈的心智模型。Python 開發者第一個會撞到的實務門檻不是語法，而是「Rust 用什麼工具建專案、裝套件、跑測試」。好消息是：Rust 只用一個工具 `cargo` 就包辦全部，不用像 Python 還要協調 `pip` + `venv` + `pyproject.toml`。

### 1. rustc 與 cargo：Rust 的兩大工具

- `rustc`：Rust 編譯器，把 `.rs` 原始碼編譯成執行檔。一般不會直接呼叫它，而是透過 cargo。
- `cargo`：Rust 的官方建置工具與套件管理員，**等同於 Python 的 `pip` + `venv` + `setuptools` + `pyproject.toml` 全部合一**。日常開發幾乎只用 cargo。

Python 對照表：

| Python                        | Rust                            | 說明                                                     |
| ----------------------------- | ------------------------------- | -------------------------------------------------------- |
| `pip install`                 | `cargo add` 或編輯 `Cargo.toml` | 安裝相依套件                                             |
| `venv` / `virtualenv`         | （不需要）                      | cargo 用 `target/` 隔離建置產物，全域套件庫在 `~/.cargo` |
| `python setup.py build`       | `cargo build`                   | 編譯                                                     |
| `python main.py`              | `cargo run`                     | 編譯並執行                                               |
| `pyproject.toml` / `setup.py` | `Cargo.toml`                    | 專案設定檔                                               |
| `pytest`                      | `cargo test`                    | 跑測試                                                   |
| `pip freeze`                  | `Cargo.lock`                    | 鎖定相依版本                                             |

### 2. 建立新專案：cargo new

```bash
cargo new my_project --edition 2021
```

說明：

- `--edition 2021` 指定 edition（本教程統一用 2021；不指定時 cargo 1.97 預設用 2024）。
- 產生結構：

  ```text
  my_project/
  ├── Cargo.toml      # 專案設定檔（對應 pyproject.toml）
  ├── src/
  │   └── main.rs     # 程式進入點，內含 fn main()
  └── .gitignore      # 自動忽略 target/
  ```

- `src/main.rs` 預設內容是 `fn main() { println!("Hello, world!"); }`。
- `--bin`（預設）建執行檔專案，`--lib` 建函式庫專案。

Python 對照：等同於 `python -m venv my_project` + 手寫 `pyproject.toml` + 建 `src/` 目錄，但 cargo 一個指令全包。

### 3. Cargo.toml：專案設定檔

展示一個典型的 `Cargo.toml`（以本教程的 ch01-cli 為例）：

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
```

說明各區塊：

- `[package]`：專案 metadata（名稱、版本、edition）。
- `[dependencies]`：相依套件清單，格式 `<crate名> = { version = "...", features = [...] }` 或簡寫 `<crate名> = "版本號"`。
- 對照 Python：`[package]` 對應 `pyproject.toml` 的 `[project]`，`[dependencies]` 對應 `dependencies = [...]`。

#### 什麼是 `features = ["derive"]`？

`features = ["derive"]` 常在 `serde`、`clap`、`tokio` 這類 crate 看到，是 Rust 套件管理獨有的觀念，Python 沒有直接對應物。一句話：**features 是套件作者提供的「選用功能開關」**，你開了才會啟用某些功能（通常是 derive 巨集）。Rust 強調「不用就不付代價」--把少用的功能放在 feature flag 後面，預設不編譯，能加快編譯、縮小執行檔。

以 `serde = { version = "1", features = ["derive"] }` 為例：

- `serde` 是 Rust 最常用的序列化 crate（類似 Python 的 `json` 模組，但更強大、型別安全）。
- 序列化需要幫每個 struct 實作 `Serialize` trait。**兩種做法**：
  1. **手動實作**：自己寫 `impl Serialize for User`，告訴 serde 怎麼把每個欄位轉成 JSON。要寫約 10 行樣板碼。
  2. **derive 巨集**：在 struct 上加 `#[derive(Serialize)]`，編譯時巨集自動產生那些樣板碼。
- `features = ["derive"]` 就是「啟用 derive 巨集這個功能」。不加這行，`#[derive(Serialize)]` 會編譯失敗（巨集沒被引入）；加了之後，下面這樣寫就會自動產生實作：

```rust
use serde::Serialize;

#[derive(Serialize)]   // derive 巨集產生 impl Serialize for User
struct User { name: String, age: u32 }
```

對比手動實作（同樣的效果，但全自己寫）：

```rust
use serde::{Serialize, Serializer};

struct User { name: String, age: u32 }

impl Serialize for User {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = ser.serialize_struct("User", 2)?;
        st.serialize_field("name", &self.name)?;
        st.serialize_field("age", &self.age)?;
        st.end()
    }
}
```

兩者跑出來的 JSON 完全一樣（`{"name":"小明","age":28}`），但 derive 寫法只要 3 行，手動寫法要 12 行。**features 的代價**：啟用 derive 會多編譯 `serde_derive` 這個 proc-macro crate（首次編譯稍慢），換來之後每個 struct 省 10 行樣板。實務上幾乎所有專案都會開 `derive`。

Python 對照：Python 沒有這種「選用功能」觀念，`pip install serde` 就是全裝。Rust 的 features 比較接近 `pip install "fastapi[all]"` 的 extras，但更細粒度、而且影響編譯結果。本教程的 ch01-cli 用 `clap = { features = ["derive"] }`，ch06-web 用 `serde = { features = ["derive"] }`，都是同樣的道理。

### 4. 日常開發指令

| 指令                     | 作用                                   | Python 對照                   |
| ------------------------ | -------------------------------------- | ----------------------------- |
| `cargo build`            | 編譯專案（產出在 `target/debug/`）     | `python -m build`             |
| `cargo run`              | 編譯並執行 `src/main.rs`               | `python main.py`              |
| `cargo run -- arg1 arg2` | 執行時傳命令列參數                     | `python main.py arg1 arg2`    |
| `cargo check`            | 只做型別檢查不產執行檔（比 build 快）  | `mypy .`                      |
| `cargo test`             | 執行所有 `#[test]` 函數                | `pytest`                      |
| `cargo build --release`  | 最佳化編譯（產出在 `target/release/`） | `python -O main.py`（概念上） |

重要觀念：

- 第一次 `cargo build` 會產生 `Cargo.lock`（鎖定相依確切版本，對應 `pip freeze` 的 `requirements.txt`）。**binary 專案要 commit `Cargo.lock`，library 專案通常不 commit**。
- `target/` 是建置產出，**不入版控**（`cargo new` 自動產生 `.gitignore` 忽略它）。

### 5. 相依套件管理

**方法一：cargo add（推薦）**

```bash
cargo add serde --features derive
```

cargo 會自動抓最新版、寫入 `Cargo.toml` 的 `[dependencies]`，並更新 `Cargo.lock`。輸出範例：

```
Adding serde v1.0.229 to dependencies
```

**方法二：手動編輯 Cargo.toml**

在 `[dependencies]` 加一行：

```toml
serde = { version = "1", features = ["derive"] }
```

然後 `cargo build`，cargo 會自動抓取。
（至於 `features = ["derive"]` 是什麼意思，見前面〈什麼是 `features = ["derive"]`？〉小節的詳細說明。）

Python 對照：`cargo add` 之於手動編輯 `Cargo.toml`，就像 `pip install` 之於手動編 `requirements.txt`--差別是 cargo 不需要虛擬環境，每個專案的相依天然隔離在各自的 `Cargo.lock`。

### 6. cargo doc：產生文件

```bash
cargo doc --no-deps --open
```

- 為你的專案產生 HTML API 文件（在 `target/doc/`）。
- `--no-deps` 只產生自己的 crate，不含相依套件（加快速度）。
- `--open` 產完自動用瀏覽器開啟。
- Python 對照：類似 `pdoc` 或 `sphinx`，但 cargo 內建、零設定。

### 7. 本教程的 workspace 結構

本教程不是單一專案，而是一個 **cargo workspace**：根 `Cargo.toml` 用 `[workspace]` 列出所有成員 crate，各章在 `chapters/chXX-xxx/`。所以從根目錄要用 `cargo run -p ch00-basics` 指定要跑哪一章，或進入單章目錄用 `cargo run`。這對應到 Python 的 monorepo 概念（多個子專案共用一個根設定）。

## Python 對照

| Python                       | Rust                                    | 說明                                   |
| ---------------------------- | --------------------------------------- | -------------------------------------- |
| `def add(a, b): ...`         | `fn add(a: i32, b: i32) -> i32 { ... }` | Rust 需標注型別；最後表達式即回傳值    |
| `try / except`               | `Result<T, E>` + `match`                | Rust 的錯誤是「值」不是「例外」        |
| `list`                       | `Vec<T>`                                | 可增長的陣列                           |
| `dict`                       | `HashMap<K, V>`                         | 鍵值對應                               |
| （沒有所有權概念，靠 GC）    | ownership / borrow                      | Rust 靠所有權規則管理記憶體，不用 GC   |
| `class`                      | `struct` + `impl`                       | struct 放資料，impl 放方法             |
| duck typing（鴨子型別）      | `trait`                                 | Rust 靠 trait 做多型，編譯期檢查       |
| `match`（3.10+）或 `if/elif` | `match`                                 | Rust 的 match 更強大，可配對範圍、解構 |

## 概念講解

### 1. 變數與可變性

在 Python 裡，所有變數都可以重新賦值——你寫 `x = 5` 之後再寫 `x = 6` 完全正常。

Rust 則相反：**變數預設是不可變的（immutable）**。`let x = 5;` 之後，你不能再修改 `x`。如果需要可變，要明確寫 `let mut y = 5;`。

這個設計是為了安全性：編譯器能保證一個不可變的值不會在任何地方被偷偷改掉。Rust 鼓勵你「預設不可變，需要時才打開 mut」，而 Python 是「永遠可變」。

### 2. 基本型別

Rust 是**靜態型別**語言，每個值在編譯期就有確定型別；Python 是動態型別。

幾個關鍵差異：

- **整數**：`i32`（有號 32 位元）、`u32`（無號）等，Python 只有 `int`（任意精度）。
- **浮點數**：`f64`，對應 Python 的 `float`。
- **布林**：`bool`，兩者相同。
- **字串**：這是最大的差異。Rust 有兩種字串型別：
  - `&str`：字串切片（string slice），是對某處字串資料的借用，不可變。
  - `String`：擁有所有權、可增長、可修改的字串。
  - Python 不區分這兩者，只有 `str`。
- **集合**：`Vec<T>` 對應 `list`，`HashMap<K, V>` 對應 `dict`。
- **Vec 的常用方法**（練習會用到）：`.is_empty()` 判斷是否為空、`.len()` 取長度、`.iter().sum()` 算總和、`vec[0]` 取第一個元素（越界會 panic，安全取值用 `.get(0)` 回傳 `Option`）。
- **型別轉換（casting）**：Rust 不會自動把 `i32` 當 `f64` 用，要明確寫 `as f64` 轉換，例如 `sum as f64 / nums.len() as f64`。Python 會隱式轉換（`3 / 2 == 1.5`），Rust 要求顯式。

### 3. 函數

Python 的 `def add(a, b): return a + b` 對應 Rust 的：

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

兩個差異：

1. **型別標注**：Rust 必須標注參數型別（`a: i32`）和回傳型別（`-> i32`）；Python 只能用型別提示（optional）。
2. **回傳值**：Rust 函數最後一個「表達式」（沒有分號）就是回傳值，不需寫 `return`。你也可以寫 `return a + b;`，但慣例上省略。

### 4. Ownership & Borrowing（重點）

這是 Rust 最獨特、也是 Python 開發者最需要適應的概念。Rust **沒有垃圾回收（GC）**，而是靠**所有權規則**在編譯期管理記憶體。

**三條規則：**

1. 每個值在任一時刻都有**一個**擁有者（owner）。
2. 當擁有者離開作用域，值就被釋放（drop）。
3. 把值賦給另一個變數或傳進函數時，所有權**轉移（move）**，原變數失效。

**所有權轉移（move）：**

```rust
let s1 = String::from("hi");
let s2 = s1; // 所有權轉移給 s2，s1 從此失效
// println!("{}", s1); // 編譯錯誤！
```

Python 沒有這個概念：`s2 = s1` 只是把兩個名字指向同一個物件，兩個都能用（靠引用計數 GC 決定何時釋放）。

**借用（borrow）：**

如果不想轉移所有權，就傳「參考」`&`：

```rust
fn str_len(s: &String) -> usize { s.len() }
let len = str_len(&s2); // 借用，s2 仍可用
```

對照 Python：Python 變數本來就是引用，所以沒有「借不借用」的分別。

**可變借用（mutable borrow）：**

要修改借用的值，用 `&mut`：

```rust
fn add_bang(s: &mut String) { s.push('!'); }
let mut s3 = String::from("hi");
add_bang(&mut s3); // s3 變成 "hi!"
```

**借用規則：** 同一時間，對同一資料，**要嘛有多個不可變借用 `&`，要嘛有一個可變借用 `&mut`**，兩者不可同時存在。這避免資料競爭（data race），是 Rust 記憶體安全的核心。

### 5. 錯誤處理

Python 用 `try/except` 捕捉例外。Rust **沒有例外**——錯誤是普通的**值**，用 `Result` 型別表示：

```rust
enum Result<T, E> {
    Ok(T),   // 成功，帶值 T
    Err(E),  // 失敗，帶錯誤 E
}
```

用 `match` 處理：

```rust
match parse_num("42") {
    Ok(n) => println!("成功: {}", n),
    Err(e) => println!("失敗: {}", e),
}
```

好處：編譯器會逼你處理 `Err` 的情況，不會像 Python 那樣「忘了 except」就讓程式崩潰。`?` 運算子可以簡化「成功就取值、失敗就提早回傳」的寫法。

**Option：可能沒有值**

`Result` 處理「成功或失敗」，而 `Option` 處理「有或沒有」--當一個函數可能不回傳值時（例如在 `Vec` 裡找不到元素），用 `Option` 表示。它的結構與 `Result` 幾乎一樣：

```rust
enum Option<T> {
    Some(T),  // 有值，帶 T
    None,     // 沒有值
}
```

Python 沒有直接對應的型別（最接近的是 `Optional[T]` 型別提示，或回傳 `None`），但 Rust 的 `Option` 是編譯期強制：回傳型別寫 `Option<T>`，呼叫端就**必須**處理 `None` 的情況，不能假裝「一定有值」。

搭配 `match` 使用：

```rust
fn first_item(nums: &Vec<i32>) -> Option<i32> {
    if nums.is_empty() {
        None
    } else {
        Some(nums[0])
    }
}

match first_item(&vec) {
    Some(n) => println!("第一個元素: {}", n),
    None => println!("空的 Vec"),
}
```

這正是「練習」會用到的型別--算平均值時，空 `Vec` 沒有平均值，所以回傳 `Option<f64>`：有值回 `Some(平均)`，空回 `None`。

### 6. Struct + impl 與 Enum（取代 class/OOP）

Rust 沒有 `class`，但你能用 `struct`（資料）+ `impl`（方法）+ `enum`（多型）做出 OOP 的效果，而且更安全。

#### 6.1 Struct + impl = class（資料 + 方法）

Python 把資料和方法都包在 `class` 裡；Rust 把兩者分開：`struct` 定義欄位，`impl` 定義方法。

**Python class：**

```python
class User:
    def __init__(self, name, age):
        self.name = name
        self.age = age
    def greeting(self):
        return f"我是 {self.name}，{self.age} 歲"
    def have_birthday(self):
        self.age += 1
```

**Rust struct + impl：**

```rust
struct User { name: String, age: u32 }

impl User {
    // 關聯函數（無 self）：對應 @staticmethod，慣例用 new() 當建構子
    fn new(name: &str, age: u32) -> User {
        User { name: String::from(name), age }
    }
    // &self 方法：唯讀，對應 Python 的 def method(self)
    fn greeting(&self) -> String {
        format!("我是 {}，{} 歲", self.name, self.age)
    }
    // &mut self 方法：可修改 self，對應會改屬性的 Python 方法
    fn have_birthday(&mut self) {
        self.age += 1;
    }
}
```

三種 `self` 的差別：

| 寫法 | 意思 | Python 對應 |
|------|------|------------|
| `&self` | 不可變借用，唯讀不改 | `def method(self)` 只讀 |
| `&mut self` | 可變借用，可改欄位 | `def method(self)` 會改 `self.xxx` |
| `self`（少用） | 取得所有權，呼叫後原物件失效 | （無對應） |
| 無 self | 關聯函數，用 `Type::func()` 呼叫 | `@staticmethod` |

呼叫方式：

```rust
let mut user = User::new("小明", 28);   // 關聯函數用 :: 呼叫
println!("{}", user.greeting());         // 方法用 . 呼叫
user.have_birthday();                    // &mut self 會改 user
```

#### 6.2 Enum 帶資料 = 多型（取代繼承）

Python 做形狀多型通常用繼承；Rust 用 **enum variant 帶資料 + match 分派**。

**Python 繼承多型：**

```python
import math
class Shape:
    def area(self): pass
class Circle(Shape):
    def __init__(self, r): self.r = r
    def area(self): return math.pi * self.r ** 2
class Rectangle(Shape):
    def __init__(self, w, h): self.w, self.h = w, h
    def area(self): return self.w * self.h
```

**Rust enum + match 分派：**

```rust
enum Shape {
    Circle(f64),           // variant 帶一個 f64（半徑）
    Rectangle(f64, f64),   // variant 帶兩個 f64（寬、高）
    Square(f64),           // variant 帶一個 f64（邊長）
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => 3.14 * r * r,       // r 是從 variant 取出的值
            Shape::Rectangle(w, h) => w * h,
            Shape::Square(s) => s * s,
        }
    }
    fn name(&self) -> &str {
        match self {
            Shape::Circle(_) => "圓形",
            Shape::Rectangle(_, _) => "矩形",
            Shape::Square(_) => "正方形",
        }
    }
}
```

關鍵差異：

- `Shape::Circle(r)` 在 `match` 裡同時做兩件事：**判斷是哪個 variant** + **取出內部資料**（`r` = 半徑），一步到位。
- 不用 `r` 時用 `_` 忽略（如 `Shape::Circle(_)`）。
- 編譯器檢查 `match` 是否**窮盡**所有 variant--新增 variant 時所有 match 都會報錯，逼你處理新情況（Python 繼承不會提醒你）。
- 一個 `enum` 就是「這個值可能是 Circle、Rectangle 或 Square」，比繼承更輕量，不需要 vtable。

#### 6.3 OOP 觀念對照總表

| Python OOP | Rust | 說明 |
|---|---|---|
| `class User:` | `struct User { ... }` | 資料欄位 |
| `def __init__(self, ...)` | `fn new(...) -> User`（關聯函數） | 建構子 |
| `def method(self, ...)` | `fn method(&self, ...)` | 唯讀方法 |
| 會改屬性的方法 | `fn method(&mut self, ...)` | 可變方法 |
| `@staticmethod` | `fn func(...)`（無 self） | 關聯函數 |
| `obj.method()` | `obj.method()` | 呼叫方式相同 |
| `class Circle(Shape):` 繼承 | `enum Shape { Circle(...), ... }` | 多型用 enum 不用繼承 |
| `isinstance(x, Circle)` | `match x { Shape::Circle(_) => ... }` | 型態判斷 |
| `__str__` / `__repr__` | `#[derive(Debug)]` + `{:?}` | 印出格式 |
| `__eq__` | `#[derive(PartialEq)]` + `==` | 相等比較 |

#### 6.4 trait：Rust 的「介面」

Rust 還有 `trait`，對應 Python 的 `abc.ABC` / `typing.Protocol` / duck typing 的正式版：定義「一群型別共同有的行為」。例如 `Display` trait 定義「怎麼印出這個型別」，任何實作 `Display` 的型別都能用 `{}` 格式化。

`trait` 與 `enum + match` 互補：enum 適合「有限的已知變體」（如 Shape 只有圓/矩/方），trait 適合「開放擴展」（任何人都能為自己的型別實作 trait）。本章先不深入 trait 實作，但後續章節會大量看到 `#[derive(Debug)]`、`#[derive(Serialize)]` 等 derive--那些背後都是 trait。

### 7. Pattern matching

Rust 的 `match` 遠比 Python 3.10+ 的 `match` 強大，可以：

- **配對範圍**：`1..=12 => "上午"`
- **配對 enum variant**：`Shape::Circle(r) => ...`（同時取出變數資料）
- **解構 struct**：`let User { name, age } = &user;`
- **解構 tuple、巢狀結構**等

`match` 必須**窮盡（exhaustive）**——所有可能都要涵蓋，可用 `_` 作為萬用兜底。這保證你不會漏掉任何情況。

## 程式碼解析

以下是 `src/main.rs` 的逐段說明。

### 函數定義（模組層）

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

函數需標型別，`a + b` 沒有分號，是回傳值。對照 Python `def add(a, b): return a + b`。

```rust
fn str_len(s: &String) -> usize {
    s.len()
}
```

參數是 `&String`（參考），呼叫時傳 `&s2` 不會轉移所有權。

```rust
fn add_bang(s: &mut String) {
    s.push('!');
}
```

`&mut` 表示可變借用，能修改借來的值。

```rust
fn parse_num(s: &str) -> Result<i32, std::num::ParseIntError> {
    s.parse()
}
```

回傳 `Result`，成功是 `i32`，失敗是解析錯誤。`s.parse()` 會嘗試把字串轉成目標型別。

```rust
#[derive(Debug)]
struct User { name: String, age: u32 }

impl User {
    fn new(name: &str, age: u32) -> User { ... }
    fn greeting(&self) -> String { ... }
    fn have_birthday(&mut self) { self.age += 1; }
}

enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Square(f64),
}
```

`struct User` 放資料欄位，`impl User` 放方法。`fn new()` 是關聯函數（無 `self`），用 `User::new()` 呼叫當建構子。`&self` 是唯讀方法，`&mut self` 可改欄位。`Shape` 是帶資料的 enum，每個 variant 夾帶不同的值（半徑、寬高、邊長）。完整程式碼見 `src/main.rs`。

### 第 1 段：變數與可變性

```rust
let x = 5;          // 不可變
let mut y = 5;      // 可變
y = 6;
println!("x = {}（不可變）, y = {}（可變）", x, y);
```

`x` 不可改，`y` 加了 `mut` 才能改。

### 第 2 段：基本型別

```rust
let n: i32 = 42;
let pi: f64 = 3.14;
let flag: bool = true;
let slice: &str = "字串切片";
let mut owned = String::from("擁有的");
owned.push_str(" 字串");
let numbers: Vec<i32> = vec![1, 2, 3];
let mut scores: HashMap<&str, i32> = HashMap::new();
scores.insert("Alice", 90);
```

展示了整數、浮點數、布林、兩種字串、`Vec`、`HashMap`。注意 `String` 可以用 `.push_str()` 增長，`&str` 不行。

### 第 3 段：函數

```rust
let sum = add(3, 4);
println!("add(3, 4) = {}", sum);
```

直接呼叫前面定義的 `add`。

### 第 4 段：Ownership & Borrowing

```rust
let s1 = String::from("hi");
let s2 = s1;  // 所有權轉移，s1 失效
println!("所有權轉移後 s2 = \"{}\"", s2);

let len = str_len(&s2);  // 借用，s2 仍可用
println!("借用 str_len(&s2) = {}", len);

let mut s3 = String::from("hi");
add_bang(&mut s3);  // 可變借用
println!("可變借用後 s3 = \"{}\"", s3);
```

這段示範了 move、borrow、mutable borrow 三種行為。記住規則：多個 `&` **或**一個 `&mut`，不可同時。

### 第 5 段：錯誤處理

```rust
for input in ["42", "abc"] {
    match parse_num(input) {
        Ok(n) => println!("parse_num(\"{}\") = Ok({})", input, n),
        Err(e) => println!("parse_num(\"{}\") = Err({})", input, e),
    }
}
```

`"42"` 解析成功回 `Ok(42)`，`"abc"` 失敗回 `Err(...)`。`match` 同時處理兩種情況。

### 第 6 段：Struct + impl 與 Enum（取代 class/OOP）

```rust
let mut user = User::new("小明", 28);   // 關聯函數，用 :: 呼叫（建構子）
println!("{}", user.greeting());         // &self 方法，用 . 呼叫
user.have_birthday();                    // &mut self 方法，會改 user.age
println!("過生日後: {}", user.greeting());
```

`User::new()` 是關聯函數（無 `self`），用 `Type::func()` 呼叫，對應 Python `User("小明", 28)`。`greeting()` 是 `&self` 唯讀方法，`have_birthday()` 是 `&mut self` 可變方法--注意 `user` 要宣告為 `let mut` 才能呼叫 `&mut self` 方法。

```rust
let shapes = [Shape::Circle(5.0), Shape::Rectangle(3.0, 4.0), Shape::Square(2.0)];
for s in &shapes {
    println!("{} 的面積 = {:.2}", s.name(), s.area());
}
```

`Shape` 是帶資料的 enum。`area()` 用 `match` 依 variant 分派：`Shape::Circle(r)` 同時判斷 variant 並取出半徑 `r`。這就是 Rust 取代繼承多型的方式--不用 `class Circle(Shape)`，用 enum variant + match。

### 第 7 段：Pattern matching

```rust
let hour = 15;
let period = match hour {
    1..=12 => "上午",
    13..=17 => "下午",
    _ => "其他",
};
```

配對數字範圍，`_` 是萬用兜底。

```rust
let User { name, age } = &user;
println!("解構 User: name = \"{}\", age = {}", name, age);
```

解構 struct，一次取出 `name` 和 `age`（這裡因為匹配的是 `&user`，`name`、`age` 會是參考）。

## 執行方式

從 workspace 根目錄執行：

```bash
cargo run -p ch00-basics
```

或進入章節目錄執行：

```bash
cd chapters/ch00-basics
cargo run
```

你會看到 7 段示範輸出，分別對應變數、型別、函數、ownership、錯誤處理、struct/enum、pattern matching。
工具鏈的詳細用法（cargo new / build / add / test 等）見前面的「Rust 工具鏈簡介」段落。

## 重點回顧

1. **Ownership 三規則**：每個值有唯一擁有者；擁有者離開作用域時值被釋放；賦值或傳參會轉移所有權（move）。這是 Rust 不靠 GC 也能管理記憶體的關鍵。
2. **Borrowing 規則**：同一時間「多個 `&`」**或**「一個 `&mut`」，兩者不可並存。
3. **Result 取代例外**：Rust 沒有 `try/except`，錯誤是 `Result` 值，用 `match` 或 `?` 處理，編譯器逼你面對失敗情況。
4. **Option 處理「有或無」**：可能不回傳值的函數用 `Option<T>`（`Some(T)` / `None`），與 `Result` 是同結構的 enum，同樣用 `match` 處理。
5. **Struct+impl 取代 class**：Rust 用 `struct` 放資料、`impl` 放方法（`&self` 唯讀、`&mut self` 可變），沒有繼承；多型用 enum variant 帶資料 + `match` 分派，不用 `class Child(Parent)`。
6. **預設不可變**：`let x = 5` 不可改，要 `let mut` 才行--這是安全性的基礎。
7. **靜態型別**：編譯期決定型別，Python 開發者要適應「先標好型別」的習慣。

## 練習

寫一個函數計算 `Vec<i32>` 的平均值，回傳 `Option<f64>`：

- 空 `Vec` 回傳 `None`（因為無法除以 0）。
- 非空 `Vec` 回傳 `Some(平均值)`。

提示：

```rust
fn average(nums: &Vec<i32>) -> Option<f64> {
    // 在這裡實作
    // 想想：怎麼判斷空 Vec？怎麼算總和？i32 轉 f64 用 `as f64`
}
```
（提示：`Option`、`is_empty()`、`iter().sum()`、`as f64` 的用法見前面〈概念講解〉第 2 段與第 5 段。）

參考答案（先自己試再看）：

```rust
fn average(nums: &Vec<i32>) -> Option<f64> {
    if nums.is_empty() {
        None
    } else {
        let sum: i32 = nums.iter().sum();
        Some(sum as f64 / nums.len() as f64)
    }
}
```

這個練習同時用到借用（`&Vec`）、`Option`（與 `Result` 類似的列舉）、型別轉換（`as f64`），是綜合複習。
