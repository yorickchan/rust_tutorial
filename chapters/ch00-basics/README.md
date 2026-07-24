# 第 0 章：Rust 基礎（給 Python 開發者）

這一章是整個教程的起點。如果你已經會寫 Python，那麼 Rust 的語法對你來說不會太陌生；但 Rust 有幾個核心觀念是 Python 完全沒有的——尤其是 **ownership（所有權）**。本章會用 Python 對照的方式，帶你一次理解 Rust 的 7 個基礎概念。

## 學習目標

讀完本章後，你應該能：

- 理解 Rust 與 Python 的**核心差異**：所有權（ownership）、型別系統（type system）、錯誤處理（error handling）。
- 知道為什麼 Rust 變數**預設不可變**，以及如何用 `mut` 改變它。
- 看懂 ownership 轉移（move）、借用（borrow）、可變借用（mutable borrow）三者的差別與規則。
- 用 `Result` + `match` 取代 Python 的 `try/except`。
- 用 `struct`、`enum`、`match` 組織資料與邏輯。

## Python 對照

| Python | Rust | 說明 |
|---|---|---|
| `def add(a, b): ...` | `fn add(a: i32, b: i32) -> i32 { ... }` | Rust 需標注型別；最後表達式即回傳值 |
| `try / except` | `Result<T, E>` + `match` | Rust 的錯誤是「值」不是「例外」 |
| `list` | `Vec<T>` | 可增長的陣列 |
| `dict` | `HashMap<K, V>` | 鍵值對應 |
| （沒有所有權概念，靠 GC） | ownership / borrow | Rust 靠所有權規則管理記憶體，不用 GC |
| `class` | `struct` + `impl` | struct 放資料，impl 放方法 |
| duck typing（鴨子型別） | `trait` | Rust 靠 trait 做多型，編譯期檢查 |
| `match`（3.10+）或 `if/elif` | `match` | Rust 的 match 更強大，可配對範圍、解構 |

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

### 6. Struct 與 Enum

**Struct** 對照 Python 的 `class`（資料部分）：

```rust
struct User {
    name: String,
    age: u32,
}
```

Rust 把「資料」和「方法」分開：資料放 `struct`，方法放 `impl` 區塊。Python 是兩者合在 `class` 裡。

**Enum** 是 Rust 的列舉，但比 Python 的 `enum` 模組強大得多——每個 variant 還可以帶資料（本章先示範不帶資料的最簡形式）：

```rust
enum Direction { Up, Down, Left, Right }
```

`match` 是搭配 enum 的天然工具，能對每個 variant 做不同處理，而且編譯器會檢查你是否涵蓋所有 variant。

### 7. Pattern matching

Rust 的 `match` 遠比 Python 3.10+ 的 `match` 強大，可以：

- **配對範圍**：`1..=12 => "上午"`
- **配對 enum variant**：`Direction::Up => ...`
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

enum Direction { Up, Down, Left, Right }
```
`#[derive(Debug)]` 讓 struct 可用 `{:?}` 印出。`Direction` 是不帶資料的 enum。

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

### 第 6 段：Struct 與 Enum

```rust
let user = User { name: String::from("小明"), age: 28 };
let dir = Direction::Down;
println!("User: {:?}", user);
match dir {
    Direction::Up => println!("Direction::Up => 上"),
    Direction::Down => println!("Direction::Down => 下"),
    Direction::Left => println!("Direction::Left => 左"),
    Direction::Right => println!("Direction::Right => 右"),
}
```
建立 `User` 實例並用 `{:?}` 印出；用 `match` 對 `Direction` 的每個 variant 做不同處理。

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
let d = Direction::Right;
let arrow = match d {
    Direction::Up => "↑",
    Direction::Down => "↓",
    Direction::Left => "←",
    Direction::Right => "→",
};
```
配對 enum variant。

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

## 重點回顧

1. **Ownership 三規則**：每個值有唯一擁有者；擁有者離開作用域時值被釋放；賦值或傳參會轉移所有權（move）。這是 Rust 不靠 GC 也能管理記憶體的關鍵。
2. **Borrowing 規則**：同一時間「多個 `&`」**或**「一個 `&mut`」，兩者不可並存。
3. **Result 取代例外**：Rust 沒有 `try/except`，錯誤是 `Result` 值，用 `match` 或 `?` 處理，編譯器逼你面對失敗情況。
4. **預設不可變**：`let x = 5` 不可改，要 `let mut` 才行——這是安全性的基礎。
5. **靜態型別**：編譯期決定型別，Python 開發者要適應「先標好型別」的習慣。

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
