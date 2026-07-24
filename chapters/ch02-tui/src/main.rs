// 第 2 章：終端介面（ratatui + crossterm）
// 互動式計數器 TUI：按 ↑/k 增加、↓/j 減少、q 離開

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

fn main() -> std::io::Result<()> {
    // 1. 進入 raw mode：關閉終端機的列處理，讓我們能逐鍵讀取輸入
    enable_raw_mode()?;
    let mut stdout = stdout();
    // 2. 進入 alternate screen：在獨立的「螢幕緩衝區」繪製，離開後還原原畫面
    execute!(stdout, EnterAlternateScreen)?;

    // 3. 建立 ratatui terminal，底層用 crossterm 作為繪製後端
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut counter: i32 = 0;

    // 4. 把主迴圈放進 run_app，確保無論 Ok 或 Err 都會執行清理
    let result = run_app(&mut terminal, &mut counter);

    // 5. 清理（保證執行）：離開 alternate screen 並關閉 raw mode
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

/// 主事件迴圈：draw -> poll -> read -> 處理按鍵 -> 重複。
/// 把迴圈獨立出來，這樣即使中途發生錯誤，main 仍會先做完清理再回傳。
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    counter: &mut i32,
) -> std::io::Result<()> {
    loop {
        // 每幀重新繪製畫面；閉包取得 frame 交給 ui 函數
        terminal.draw(|frame| ui(frame, *counter))?;

        // 非阻塞等待事件：250ms 內沒事件就 poll 回 false，直接 continue 重新繪製
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        // 讀取事件，只處理按鍵事件
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

/// 繪製函數：用 widget 組合出畫面。
/// 注意：ratatui 0.30 用 `frame.area()` 取得繪製區域，不是舊版的 `frame.size()`。
fn ui(frame: &mut Frame, counter: i32) {
    // Block：外框 + 標題
    let block = Block::default().borders(Borders::ALL).title("計數器");
    // Paragraph：文字內容，套上 block 外框
    let paragraph = Paragraph::new(format!(
        "計數: {}\n\n按 ↑/k 增加，↓/j 減少，q 離開",
        counter
    ))
    .block(block);
    // 把 widget 渲染到整個 frame 區域
    frame.render_widget(paragraph, frame.area());
}
