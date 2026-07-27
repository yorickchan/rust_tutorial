// Rust 基礎（給 Python 開發者）
// 這支程式展示 7 個核心概念，每段以 `// === N. <標題> ===` 分隔。
// 註解會標示對應的 Python 概念，幫助你建立心智模型。

use std::collections::HashMap;

// 對照 Python：`def add(a, b): return a + b`
// Rust 需標注參數與回傳型別；函數最後一個「表達式」（沒有分號）即回傳值。
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// 借用（borrow）：參數是 `&String`（參考），不取走所有權，只讀取。
// 對照 Python：沒有此概念，Python 一律是引用。
fn str_len(s: &String) -> usize {
    s.len()
}

// 可變借用（mutable borrow）：參數是 `&mut String`，可修改借用來的值。
fn add_bang(s: &mut String) {
    s.push('!');
}

// 錯誤處理：回傳 `Result`，錯誤是「值」而不是「例外」。
// 對照 Python：`try/except`。
fn parse_num(s: &str) -> Result<i32, std::num::ParseIntError> {
    s.parse()
}

// Struct + impl：對照 Python 的 class（資料 + 方法）。
// `#[derive(Debug)]` 讓我們可以用 `{:?}` 印出內容。
#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

// impl 區塊：放「方法」。這就是 Python class 裡的 def 方法。
impl User {
    // 關聯函數（associated function）：沒有 self，類似 Python 的 @staticmethod。
    // 慣例用 new() 當建構子（constructor）。
    fn new(name: &str, age: u32) -> User {
        User {
            name: String::from(name),
            age,
        }
    }

    // 方法：&self = 不可變借用 self（唯讀），對應 Python 的 def method(self)。
    fn greeting(&self) -> String {
        format!("我是 {}，{} 歲", self.name, self.age)
    }

    // 方法：&mut self = 可變借用 self（可修改），對應 Python 裡會改 self 屬性的方法。
    fn have_birthday(&mut self) {
        self.age += 1;
    }
}

// Enum 帶資料：每個 variant 可以夾帶不同的資料。
// 這是 Rust 做「多型」的方式--不用繼承，用 enum + match 依 variant 分派。
// 對照 Python：Python 用 class Circle(Shape) 繼承多型，Rust 用 enum variant 取代。
#[allow(dead_code)]
enum Shape {
    Circle(f64),           // 帶一個 f64（半徑）
    Rectangle(f64, f64),   // 帶兩個 f64（寬、高）
    Square(f64),           // 帶一個 f64（邊長）
}

impl Shape {
    // 用 match 依 variant 做不同計算--這就是 Rust 的「多型分派」。
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => 3.14 * r * r,
            Shape::Rectangle(w, h) => w * h,
            Shape::Square(s) => s * s,
        }
    }

    // 回傳形狀名稱
    fn name(&self) -> &str {
        match self {
            Shape::Circle(_) => "圓形",
            Shape::Rectangle(_, _) => "矩形",
            Shape::Square(_) => "正方形",
        }
    }
}

fn main() {
    // === 1. 變數與可變性 ===
    // Python 所有變數都可重新賦值；Rust 預設「不可變」（immutable）。
    let x = 5; // 不可變，之後不能再修改 x
    let mut y = 5; // 加 `mut` 才可修改
    println!("=== 1. 變數與可變性 ===");
    println!("x = {}（不可變）, y = {}（可變）", x, y);
    y = 6; // 因為 y 宣告為 mut，這裡可以重新賦值
    println!("重新賦值後 y = {}（已從 5 改為 6）", y);
    println!();

    // === 2. 基本型別 ===
    // Python 不區分 str 型別；Rust 區分 `&str`（字串切片，借用）與 `String`（擁有、可增長）。
    let n: i32 = 42;
    let pi: f64 = 3.14;
    let flag: bool = true;
    let slice: &str = "字串切片";
    let mut owned = String::from("擁有的"); // String 可增長
    owned.push_str(" 字串");
    let numbers: Vec<i32> = vec![1, 2, 3]; // 對照 Python list
    let mut scores: HashMap<&str, i32> = HashMap::new(); // 對照 Python dict
    scores.insert("Alice", 90);
    scores.insert("Bob", 85);
    println!("=== 2. 基本型別 ===");
    println!("i32: {}, f64: {}, bool: {}", n, pi, flag);
    println!("&str: \"{}\", String: \"{}\"", slice, owned);
    println!("Vec<i32>: {:?}, HashMap: {:?}", numbers, scores);
    println!();

    // === 3. 函數 ===
    let sum = add(3, 4);
    println!("=== 3. 函數 ===");
    println!("add(3, 4) = {}", sum);
    println!();

    // === 4. Ownership & Borrowing ===（重點段）
    // 所有權轉移（move）：`let s2 = s1;` 之後 s1 失效。
    // 對照 Python：沒有此概念（Python 用引用計數 GC，多個變數可同時指向同一物件）。
    let s1 = String::from("hi");
    let s2 = s1; // s1 的所有權「轉移」給 s2，s1 不再可用
    // println!("{}", s1); // 編譯錯誤！s1 已失效
    println!("=== 4. Ownership & Borrowing ===");
    println!("所有權轉移後 s2 = \"{}\"（s1 已失效，無法再使用）", s2);

    // 借用：傳 `&s2` 不轉移所有權，s2 仍可用。
    let len = str_len(&s2);
    println!("借用 str_len(&s2) = {}", len);

    // 可變借用：需 `&mut`，且被借用的變數本身也要是 `mut`。
    let mut s3 = String::from("hi");
    add_bang(&mut s3);
    println!("可變借用後 s3 = \"{}\"", s3);
    // 規則摘要：同一時間「要嘛多個 &（不可變借用），要嘛一個 &mut（可變借用）」，不可同時存在。
    println!();

    // === 5. 錯誤處理 ===
    // Python 用 try/except；Rust 的錯誤是「值」（Result 的 Err），用 match 處理。
    println!("=== 5. 錯誤處理 ===");
    for input in ["42", "abc"] {
        match parse_num(input) {
            Ok(n) => println!("parse_num(\"{}\") = Ok({})", input, n),
            Err(e) => println!("parse_num(\"{}\") = Err({})", input, e),
        }
    }
    println!();

    // === 6. Struct + impl 與 Enum（取代 class/OOP）===
    // Python 用 class 把資料和方法包在一起；Rust 用 struct（資料）+ impl（方法）分開寫。
    let mut user = User::new("小明", 28);   // 關聯函數（建構子），用 Type::func() 呼叫
    println!("=== 6. Struct + impl 與 Enum ===");
    println!("{}", user.greeting());         // 呼叫 &self 方法
    user.have_birthday();                    // 呼叫 &mut self 方法（會改 user）
    println!("過生日後: {}", user.greeting());

    // Enum 帶資料 + match 分派：Rust 的「多型」不靠繼承，靠 enum variant + match。
    let shapes = [
        Shape::Circle(5.0),
        Shape::Rectangle(3.0, 4.0),
        Shape::Square(2.0),
    ];
    for s in &shapes {
        println!("{} 的面積 = {:.2}", s.name(), s.area());
    }
    println!();

    // === 7. Pattern matching ===
    // Python 3.10+ 有 match，或用 if/elif；Rust 的 match 更強大（可配對範圍、解構）。
    let hour = 15;
    let period = match hour {
        1..=12 => "上午",
        13..=17 => "下午",
        _ => "其他",
    };
    println!("=== 7. Pattern matching ===");
    println!("hour = {} => {}", hour, period);

    // 解構 struct：一次取出 name 和 age（match 匹配 &user，取出的會是參考）
    let User { name, age } = &user;
    println!("解構 User: name = \"{}\", age = {}", name, age);
}
