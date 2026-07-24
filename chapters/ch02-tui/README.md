# 第 2 章：終端介面（ratatui）

## 學習目標

- 學會用 `ratatui` 在終端機繪製圖形介面（外框、文字 widget）。
- 理解終端機的「事件迴圈」：繪製 -> 等待事件 -> 讀取按鍵 -> 處理 -> 重複。
- 認識 raw mode 與 alternate screen，以及為什麼它們是 TUI 必備的兩個模式。
- 建立「資源必須清理」的直覺：終端機是有限狀態資源，離開時一定要還原。

## 本章相依套件與 Cargo.toml

本章會用到 2 個 crate：`ratatui`（TUI 繪圖框架）與 `crossterm`（終端機底層控制）。完整 `Cargo.toml`：

```toml
[package]
name = "ch02-tui"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.30"
crossterm = "0.29"

[[bin]]
name = "ch02-tui"
path = "src/main.rs"
```

### 各套件用途與 features 說明

| crate | 用途 | Python 對照 | 為什麼選它 |
|---|---|---|---|
| `ratatui` | 高階 TUI 框架（widget 系統：外框、段落、列表） | `curses` + 手動管理游標 | 把「畫面佈局」抽象成 widget 組合，不必算座標 |
| `crossterm` | 終端機底層控制（raw mode、事件讀取、清除畫面） | `curses` 的底層部分 | 跨平台（Windows/Linux/macOS 都能用），是 ratatui 預設後端 |

### 為什麼兩個 crate 分開？

這是 Rust 生態常見的「關注點分離」設計：`ratatui` 只負責「畫什麼」（widget 佈局、渲染緩衝區），`crossterm` 只負責「怎麼畫到終端機」（進出 raw mode、讀按鍵、送 ANSI escape sequence）。兩者透過 ratatui 的 `Backend` trait 合作。Python 的 `curses` 把這兩層混在一起，所以 Python 開發者一開始會覺得「為什麼要裝兩個」--答案是各司其職、可替換後端（例如換成 `termion` 只需改一行）。

### features 開關說明

- **`ratatui = "0.30"`**：本章只用基礎 widget（`Block` 外框、`Paragraph` 段落），不需額外 features。`ratatui` 預設就啟用 `crossterm` 後端整合（`feature "crossterm"` 是 default features 的一部分），所以不必寫 `features = ["crossterm"]`。如果要用進階 widget（如日曆），才需加 `features = ["all-widgets"]` 或 `features = ["widget-calendar"]`。
- **`crossterm = "0.29"`**：純版本號、無 features。本章用到 `event::read()`（讀按鍵）、`terminal::enable_raw_mode()`（進 raw mode）、`execute!`（送控制指令），全在 default features 內。

### 安裝指令對照

```bash
# 方法一：cargo add（推薦）
cargo add ratatui
cargo add crossterm

# 方法二：直接編輯 [dependencies] 區塊（如上面的 Cargo.toml 所示）
```

Python 對照：相當於 `pip install ratatui crossterm`，但 Rust 不需要虛擬環境。注意 `ratatui` 與 `crossterm` 的版本要相容--本教程用 `ratatui 0.30` + `crossterm 0.29`（這是 ratatui 0.30 測試過的組合），亂配版本可能編譯失敗。

## Python 對照

在 Python 裡，最接近的工具是標準庫的 `curses`，它同樣需要進入特殊終端模式、讀取按鍵、繪製文字。但 `ratatui` 提供了一套更高階的 **widget 系統**（外框、段落、列表、表格…），讓你像組積木一樣拼畫面，而不必手動管理游標座標。

| 概念 | Python（curses） | Rust（ratatui + crossterm） |
|---|---|---|
| 進入特殊模式 | `curses.initscr()` / `curses.noecho()` | `enable_raw_mode()` + `EnterAlternateScreen` |
| 還原終端 | `curses.endwin()` | `disable_raw_mode()` + `LeaveAlternateScreen` |
| 畫外框 | `window.border()` | `Block::default().borders(Borders::ALL)` |
| 顯示文字 | `window.addstr(...)` | `Paragraph::new(...)` |
| 讀按鍵（非阻塞） | `window.getch()` + `nodelay` | `event::poll()` + `event::read()` |
| 取得繪製大小 | `window.getmaxyx()` | `frame.area()` |

並排版碼對照（一個最簡單的「按 q 離開」迴圈）：

```python
# Python curses
import curses

def main(stdscr):
    curses.curs_set(0)          # 隱藏游標
    stdscr.clear()
    stdscr.addstr(0, 0, "按 q 離開")
    stdscr.refresh()
    while stdscr.getch() != ord('q'):
        pass

curses.wrapper(main)            # wrapper 會自動還原終端
```

```rust
// Rust ratatui + crossterm
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

// ...（進入 raw mode / alternate screen 略）...
loop {
    terminal.draw(|frame| { /* 繪製 */ })?;
    if !event::poll(Duration::from_millis(250))? { continue; }
    if let Event::Key(key) = event::read()? {
        if key.code == KeyCode::Char('q') { break; }
    }
}
// ...（離開 alternate screen / 關閉 raw mode 略）...
```

注意 Python 的 `curses.wrapper(main)` 會在函數結束（或例外）時自動呼叫 `endwin()` 還原終端，這對 Python 開發者很方便。Rust 沒有這種隱式保護，你必須自己安排清理邏輯（見下方概念講解）。

## 概念講解

### 1. 終端機模式：raw mode 與 alternate screen

平常你在 shell 輸入文字時，終端機處於「cooked mode（列處理模式）」：你打字會先顯示在螢幕上，要按 Enter 才會把整行送給程式。對 TUI 來說這不適用，因為我們希望「按一個鍵立刻反應」。

- **raw mode**：關閉列處理，讓程式能逐鍵讀取輸入（按 ↑ 就立刻收到，不必按 Enter）。`enable_raw_mode()` 就是做這件事。
- **alternate screen**：終端機有兩個螢幕緩衝區。平常的 shell 在「主螢幕」；TUI 程式進入「替代螢幕」繪製，離開時切回主螢幕，原本 shell 的畫面與捲動歷史完整保留。這就是為什麼 `vim`、`htop` 關掉後，shell 畫面不會被弄亂。

這兩個是「全域狀態改變」：一旦你開了 raw mode，整個終端機的行為都變了。如果程式在沒還原的情況下結束，shell 就會卡在奇怪狀態（例如打字不顯示、Enter 沒反應）。

### 2. 事件迴圈

TUI 的核心是一個不斷重複的迴圈：

```
繪製畫面 (draw)
   ↓
等待事件 (poll，非阻塞，最多等 250ms)
   ↓
有事件就讀取 (read)
   ↓
處理按鍵（更新狀態）
   ↓
回到繪製（下一幀）
```

關鍵是 `poll` 是**非阻塞**的：它在指定時間內有事件就回 `true`，沒事件就回 `false`。這讓我們不會卡死在「等使用者按鍵」，可以持續重繪畫面（例如顯示時鐘、動畫）。如果 `poll` 回 `false`，我們就 `continue` 重新繪製。

### 3. Widget 系統

ratatui 用 **widget** 來描述畫面元素，每個 widget 是一個資料結構，渲染時才真正畫到螢幕：

- `Block`：一個區塊，可加邊框（`Borders::ALL`）與標題（`.title(...)`）。常當容器或外框。
- `Paragraph`：一段文字，可套上 `Block` 作為外框。
- `Borders`：指定要畫哪些邊（`ALL`、`LEFT`、`TOP`…）。

widget 是**宣告式**的：你描述「我要什麼」，ratatui 負責算座標、畫字元。這比 curses 手動 `addstr(x, y, ...)` 管座標高階很多。

### 4. Frame::area() 取得繪製區域

`frame.area()` 回傳一個 `Rect`，代表整個可繪製區域的寬高。渲染 widget 時要告訴它「畫在哪裡」：

```rust
frame.render_widget(paragraph, frame.area());   // 畫滿整個畫面
```

> **版本注意**：ratatui 0.30 把舊版的 `Frame::size()` 重新命名為 `Frame::area()`（`size()` 已被標為 deprecated）。兩者回傳值相同（都是 `Rect`），只是「area」是更準確的名稱。如果你看到舊教學用 `frame.size()`，在 0.30 要改成 `frame.area()`，否則編譯會出現棄用警告甚至錯誤。

### 5. 【Python 開發者請特別注意】終端是有限狀態資源，清理是強制的

這是本章最重要的觀念。Python 開發者習慣了：

- **GC（垃圾回收）**：物件不用了自動釋放。
- **`try/finally` 與 context manager**：`with` 敘述自動清理資源，例外也會跑 `finally`。
- **例外傳播**：未捕捉的例外會一路冒上來，但程式多半仍能「結束」。

但終端機的 raw mode / alternate screen 是**外部環境的狀態**，不在 GC 管轄範圍。如果你的 Rust TUI 程式在 raw mode 中途 **panic**（或因錯誤提早 return），而清理程式碼沒被執行到，你的 shell 就會「壞掉」--打字不顯示、Enter 失靈、畫面殘留。這時只能 `reset` 或重開終端。

因此，**清理必須在所有路徑都執行**。本章用一個簡單而穩健的做法（見下方程式碼解析）：把主迴圈抽成 `run_app` 函數，`main` 裡先呼叫它、**無論結果如何都先做清理、再把結果回傳**。這樣即使迴圈內 `?` 傳播了 `Err`，清理仍會跑。

> 進階做法：可以安裝 `panic hook`，在 panic 時也執行 `disable_raw_mode` 與 `LeaveAlternateScreen`，進一步保障「panic 也能還原終端」。本章為入門保持簡單，但實務專案建議加上。

## 程式碼解析

逐段解說 `src/main.rs`。

### 匯入

```rust
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{stdout, Stdout},
    time::Duration,
};
```

- `crossterm` 負責終端機低階操作（模式切換、事件讀取），`ratatui` 負責高階繪製。兩者搭配：ratatui 是繪製層，crossterm 是後端。
- `execute!` 巨集用來把「進入/離開 alternate screen」這類命令送到終端。
- `Stdout` 型別會在 `run_app` 的簽名出現（因為 backend 包了 `Stdout`）。

### main：進入模式 -> 跑迴圈 -> 清理

```rust
fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut counter: i32 = 0;

    // 把主迴圈抽成 run_app，確保清理一定執行
    let result = run_app(&mut terminal, &mut counter);

    // 清理（無論 run_app 是 Ok 還是 Err 都會跑到這裡）
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}
```

重點在於**清理的順序**：

1. `enable_raw_mode()` 與 `EnterAlternateScreen` 先把終端切到 TUI 模式。
2. `run_app(...)` 回傳的 `result` 可能是 `Ok(())`（使用者按 q 離開）或 `Err(...)`（繪製/讀取出錯）。我們**先把它存起來，不立刻用 `?` 往外丟**。
3. 接著無條件執行 `disable_raw_mode()` 與 `LeaveAlternateScreen`--因為這兩行不在 `?` 的傳播鏈上，無論 `result` 是什麼都會跑到。
4. 最後才 `result` 把結果回傳。

如果你直接寫 `run_app(...)?`，那麼當它回 `Err` 時，`main` 會立刻 return，**跳過底下的清理**，終端就壞了。這個「先存結果、再清理、最後回傳」的小技巧就是穩健性的來源。

### run_app：事件迴圈

```rust
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    counter: &mut i32,
) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, *counter))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => *counter += 1,
                KeyCode::Down | KeyCode::Char('j') => *counter -= 1,
                KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }
    Ok(())
}
```

- `terminal.draw(|frame| ui(frame, *counter))`：每幀呼叫閉包繪製，閉包拿到 `frame` 後交給 `ui` 函數。`*counter` 把 `&mut i32` 解參考成 `i32` 傳入。
- `event::poll(Duration::from_millis(250))`：250ms 內有事件回 `true`，沒有回 `false` 就 `continue` 重繪。這是「非阻塞等待」。
- `event::read()`：讀取一個事件。用 `if let Event::Key(key)` 只處理按鍵，忽略如視窗縮放等其他事件。
- `match key.code`：`Up`/`k` 加一、`Down`/`j` 減一、`q` 離開迴圈。`|` 是 pattern 的「或」。

> 為什麼要把迴圈抽成函數？因為 `?` 會在錯誤時提早 return；若迴圈直接寫在 `main` 裡，`?` 會跳過清理。抽成 `run_app` 後，`main` 就能「拿到結果 -> 清理 -> 回傳」，把清理與迴圈解耦。

### ui：用 widget 組畫面

```rust
fn ui(frame: &mut Frame, counter: i32) {
    let block = Block::default().borders(Borders::ALL).title("計數器");
    let paragraph = Paragraph::new(format!(
        "計數: {}\n\n按 ↑/k 增加，↓/j 減少，q 離開",
        counter
    ))
    .block(block);
    frame.render_widget(paragraph, frame.area());
}
```

- `Block::default().borders(Borders::ALL).title("計數器")`：建一個四面邊框、標題為「計數器」的區塊。
- `Paragraph::new(...)`：把計數值與操作說明組成文字，再 `.block(block)` 套上外框。
- `frame.render_widget(paragraph, frame.area())`：把 widget 渲染到整個 frame 區域。**這裡用 `frame.area()`，是 ratatui 0.30 的寫法**（舊版 `frame.size()` 已 deprecated）。

## 執行方式

從 workspace 根目錄執行：

```bash
cargo run -p ch02-tui
```

執行後會看到一個帶邊框的「計數器」畫面：

- 按 `↑` 或 `k`：計數 +1
- 按 `↓` 或 `j`：計數 -1
- 按 `q`：離開程式

離開後請確認 **shell 回到正常狀態**：打字會顯示、Enter 有反應、畫面沒有殘留的邊框。如果 shell 看起來「壞掉了」，輸入 `reset` 指令（打字可能看不見，盲打後按 Enter）即可還原。正常的話不需要 `reset`。

> 若你想在 chapter 目錄內直接跑：`cd chapters/ch02-tui && cargo run`，效果相同。

## 重點回顧

- **raw mode + alternate screen** 是 TUI 的兩個必備模式：前者讓你逐鍵讀輸入，後者讓你離開後還原原畫面。兩者都是全域狀態，必須還原。
- **事件迴圈**：`draw -> poll(非阻塞) -> read -> 處理 -> 重複`。`poll` 回 `false` 就 `continue` 重繪，不會卡死。
- **widget 系統**：`Block`（外框）、`Paragraph`（文字）、`Borders`，宣告式描述畫面，ratatui 算座標。
- **`Frame::area()`**：ratatui 0.30 取得繪製區域的方法（取代舊版 `size()`）。
- **清理是強制的**：把主迴圈抽成 `run_app`，`main` 裡「先存結果 -> 無條件清理 -> 回傳」，確保即使錯誤也還原終端。Python 的 GC / `finally` 不會幫你還原終端狀態。

## 練習（選做）

1. **重置計數器**：新增一個按鍵 `r`（`KeyCode::Char('r')`），按下時把 `counter` 歸零為 `0`。記得同步更新畫面上的操作說明文字。

2. **顯示操作紀錄**：用一個 `Vec<String>` 記錄最近幾次操作（例如「+1 -> 3」「-1 -> 2」），在畫面下方用另一個 `Paragraph` 顯示出來（提示：用 `Layout` 把畫面切成上下兩塊）。

3. **panic 安全（進階）**：安裝一個 `panic hook`（`std::panic::set_hook`），在 panic 時呼叫 `disable_raw_mode()` 與 `LeaveAlternateScreen`，這樣即使程式 panic 也能還原終端。思考：為什麼單靠 `run_app` 模式還不夠？（提示：panic 不走 `?` 的 `Err` 路徑，而是直接展開堆疊。）
