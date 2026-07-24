# Rust 入門教程（給 Python 開發者）

本教程專為熟悉 Python 的開發者設計，用 **Python 對照** 的方式帶你入門 Rust。每章是一個獨立可執行的 Cargo crate，邊讀課文邊跑範例，循序漸進掌握 Rust 的核心觀念與常見應用場景。

## 前置需求

1. 安裝 Rust 工具鏈（透過 rustup）：

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. 確認安裝成功：

   ```bash
   cargo --version
   ```

   能印出版本號即可（本教程以 Rust 1.97 / edition 2021 開發）。

## 章節索引

| 章節 | 標題 | 一句話描述 |
|------|------|-----------|
| [ch00](chapters/ch00-basics/README.md) | Rust 基礎（給 Python 開發者） | ownership / borrowing / 型別 / 錯誤處理 / struct / enum / pattern matching |
| [ch01](chapters/ch01-cli/README.md) | CLI 工具（clap） | 用 clap derive 寫一個 Todo CLI，理解命令列參數解析 |
| [ch02](chapters/ch02-tui/README.md) | 終端介面（ratatui） | 用 ratatui 做互動式計數器 TUI，理解事件迴圈與終端清理 |
| [ch03](chapters/ch03-async/README.md) | 非同步程式設計（tokio） | 循序 vs 並發，理解 async/await 與 tokio runtime |
| [ch04](chapters/ch04-networking/README.md) | 網路程式設計（reqwest） | 發 HTTP 請求、用 serde 解析 JSON（對應 Python `requests`） |
| [ch05](chapters/ch05-sqlite/README.md) | SQLite 資料庫（rusqlite） | 同步 CRUD，對應 Python `sqlite3` 模組 |
| [ch06](chapters/ch06-web/README.md) | Web 開發（axum） | 壓軸章：用 axum 建 Todo REST API，整合 async + serde |

## 使用方式

每章可獨立執行。進入該章目錄直接跑：

```bash
cd chapters/ch01-cli
cargo run -- add "買牛奶"
```

或從 workspace 根目錄用 `-p` 指定套件：

```bash
cargo run -p ch01-cli -- add "買牛奶"
```

> 每章 README 的「執行方式」段落會列出該章具體的指令與預期輸出。

## 建議學習順序

```
ch00（基礎）→ ch01（CLI）→ ch02（TUI）→ ch03（async）→ ch04（networking）→ ch05（sqlite）→ ch06（web）
```

ch00 先建立 ownership / borrowing 觀念（Python 開發者最大門檻）；ch03 的 async 是 ch04 / ch06 的前置；ch06 壓軸整合前面所學。

## 環境需求

- **ch04-networking** 需要網路連線（會呼叫 `jsonplaceholder.typicode.com`）。
- **ch05-sqlite** 使用 `rusqlite` 的 `bundled` feature，會自動編譯 SQLite，**不需要** 系統安裝 libsqlite3。
- 其餘章節離線即可執行。
