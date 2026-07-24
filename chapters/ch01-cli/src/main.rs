// ch01-cli: 用 clap derive API 實作的簡單 Todo CLI
//
// 本範例展示 Rust 的命令列參數解析。對 Python 開發者來說，
// clap 的 derive API 類似 argparse，但透過 derive 巨集直接從結構
// 定義自動產生解析邏輯，型別更安全、寫法更簡潔。

use clap::{Parser, Subcommand};

// === 命令列定義 ===

// 頂層 CLI 結構。#[derive(Parser)] 讓 clap 自動產生參數解析程式碼，
// 不用手寫 parse_args。對應 Python 的 argparse.ArgumentParser。
#[derive(Parser)]
#[command(version, about = "簡單 Todo CLI")]
struct Cli {
    /// 必選子命令（add / list / done）
    // #[command(subcommand)] 表示這個欄位是一個子命令分派器，
    // 對應 Python argparse 的 add_subparsers()。
    #[command(subcommand)]
    command: Commands,
}

// 子命令以 enum 定義：每個 variant 就是一個子命令。
// 相較於 Python 的 add_parser("add") + 一個個設定參數，
// Rust 用 #[derive(Subcommand)] 一次宣告完成，編譯器還會檢查型別。
#[derive(Subcommand)]
enum Commands {
    /// 新增待辦事項
    // task: String 是「必填位置參數」(positional argument)，
    // 不是 --task。用法：cargo run -- add "買牛奶"
    Add {
        /// 待辦內容（位置參數）
        task: String,
    },
    /// 列出所有待辦事項
    List,
    /// 將指定編號的待辦標記為完成
    // id: u32 同樣是位置參數，且型別為 u32，
    // clap 會自動把字串解析成數字、解析失敗會報錯。
    Done {
        /// 待辦編號（1 起算）
        id: u32,
    },
}

// === 資料結構 ===

// 待辦項目。
#[derive(Debug)]
struct Todo {
    text: String,
    done: bool,
}

// 注意：底下的 todos 是「記憶體內」儲存——每次 cargo run 都是一個
// 全新行程，Vec 從空開始，資料不會跨執行保留。真實應用會把資料寫入
// 檔案或資料庫（參見 README 的練習題）。

fn main() {
    // Cli::parse() 從 std::env::args() 讀取命令列參數並解析。
    // 解析失敗（例如缺參數）時 clap 會自動印出錯誤與使用說明並離開。
    let cli = Cli::parse();

    // 待辦清單（記憶體內，每次執行都從空開始）
    let mut todos: Vec<Todo> = Vec::new();

    // 用 match 配對子命令——Rust 的 enum 配 match 是處理分派的慣用法。
    match cli.command {
        Commands::Add { task } => {
            // 新增：push 進 Vec，編號為 1 起算。
            let id = todos.len() + 1;
            // 把 task 直接 move 進 Todo，再從 Vec 讀回來印，省去 clone。
            todos.push(Todo {
                text: task,
                done: false,
            });
            println!("已新增: \"{}\" (編號 {})", todos[id - 1].text, id);
        }
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
        Commands::Done { id } => {
            // id 為 1 起算，轉成 0 起算的索引。
            // 用 id as usize 把 u32 轉成索引型別。
            let index = id as usize;
            if index == 0 || index > todos.len() {
                println!("錯誤: 找不到編號 {}", id);
            } else {
                let todo = &mut todos[index - 1];
                todo.done = true;
                println!("已完成: {}", todo.text);
            }
        }
    }
}
