// 第 7 章：Rust 巨集（#[...] 與 macro_rules!）
// 展示屬性巨集的使用、derive 巨集的原理、以及如何用 macro_rules! 自訂巨集。
//
// 對照 Python：Python 沒有編譯期巨集；最接近的是 decorator（執行期）與
// 字串拼接 + exec()（危險且無型別檢查）。Rust 巨集在編譯期展開、型別安全。

// === 1. 你已經用過的 #[...]：屬性與 derive 巨集 ===
// 展示 std 內建的 derive（Debug/Clone/PartialEq）與屬性（allow/test）。
// 重申：這些都是巨集，編譯器在編譯期把它們展開成程式碼。

#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

// === 2. macro_rules!：自己寫宣告巨集 ===

/// 巨集 vs 函數：巨集在「編譯期」展開，可接受不同型別、產生新程式碼；
/// 函數在「執行期」呼叫，型別固定。對照 Python：巨集像「會長出程式碼的模板」。
macro_rules! say_hello {
    () => {
        println!("hello from macro!");
    };
}

/// 帶參數的巨集：$name 是「片段（fragment）」metavariable。
/// `$name:expr` 表示匹配一個「表達式（expression）」。
macro_rules! greet {
    ($name:expr) => {
        println!("你好, {}!", $name);
    };
}

/// 重複匹配：$($x:expr),* 表示「零或多個 expr，用逗號分隔」。
/// 這是 vec! 巨集的核心手法。對照 Python：無直接對應，最接近的是 *args 展開，
/// 但巨集是在編譯期「長出程式碼」，不是執行期收集成 tuple。
macro_rules! my_vec {
    ($($x:expr),*) => {
        {
            let mut v = Vec::new();
            $( v.push($x); )*
            v
        }
    };
}

/// 條件分支：巨集可依匹配模式不同產生不同程式碼（像 match，但在編譯期）。
macro_rules! count {
    () => { 0usize };
    ($x:expr) => { 1usize };
    ($x:expr, $y:expr) => { 2usize };
}

fn main() {
    // === 1. 屬性與 derive 巨集 ===
    let p1 = Point { x: 1, y: 2 };
    let p2 = p1.clone(); // Clone 巨集產生的方法
    println!("{:?}", p1); // Debug 巨集產生的格式化
    println!("p1 == p2? {}", p1 == p2); // PartialEq 巨集產生的 ==

    // === 2. macro_rules! 巨集 ===
    say_hello!();
    greet!("小明");
    greet!(1 + 2); // 巨集接受任何 expr，不需泛型

    let nums = my_vec!(1, 2, 3, 4, 5);
    println!("my_vec! => {:?}", nums);

    println!("count!() = {}", count!());
    println!("count!(1) = {}", count!(1));
    println!("count!(1, 2) = {}", count!(1, 2));
}

// === 3. #[test] 也是屬性巨集：順帶示範測試 ===
// cargo test 會執行所有標記 #[test] 的函數。#[test] 是屬性巨集，
// 把普通函數轉成「測試 harness 可識別的測試案例」。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_clone_is_equal() {
        let p = Point { x: 5, y: 10 };
        assert_eq!(p, p.clone());
    }

    #[test]
    fn my_vec_builds_correctly() {
        assert_eq!(my_vec!(1, 2, 3), vec![1, 2, 3]);
        let empty: Vec<i32> = my_vec!();
        assert_eq!(empty, vec![]);
    }

    #[test]
    fn count_macro_branches() {
        assert_eq!(count!(), 0);
        assert_eq!(count!(9), 1);
        assert_eq!(count!(1, 2), 2);
    }
}
