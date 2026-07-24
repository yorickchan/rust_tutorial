use std::time::{Duration, Instant};

// `#[tokio::main]` 把 `async fn main` 展開成同步的 `fn main`，
// 並在其中建立 tokio 多執行緒 runtime，讓我們能在 `main` 裡使用 `.await`。
#[tokio::main]
async fn main() {
    // === 定義非同步任務 ===
    // `async fn` 不會立即執行，而是回傳一個 Future；
    // 必須被 `.await` 或 `tokio::spawn` 排程後才會真正執行。
    async fn task(name: &str, ms: u64) {
        println!("開始 {name}");
        // `tokio::time::sleep` 回傳 Future，`.await` 在此「非阻塞」等待。
        // 等待期間 runtime 可切換去執行其他 task，而非佔住整條執行緒。
        tokio::time::sleep(Duration::from_millis(ms)).await;
        println!("完成 {name}");
    }

    // === 1. 循序執行 ===
    // 逐一 `.await`：A 跑完才換 B，兩者各 1 秒，總耗時約 2 秒。
    let start = Instant::now();
    task("A", 1000).await;
    task("B", 1000).await;
    println!("循序耗時: {:?}", start.elapsed());

    println!("---");

    // === 2. 並發執行 ===
    // `tokio::spawn` 把 task 排到 runtime 上同時執行，回傳 `JoinHandle<T>`。
    // 兩個 task 各 1 秒，但同時進行，總耗時約 1 秒。
    //
    // Python 開發者注意:
    // `tokio::spawn` 要求被 spawn 的 future 及其捕獲的值都必須是 `Send`
    // （多執行緒 runtime 會把 task 在不同執行緒間移動）。
    // 因此並發共用資料要用 `Arc`/`Mutex`，不能用 `Rc`/`RefCell`。
    // Python 的 `asyncio.create_task` 沒有這個限制（GIL + 單一執行緒）。
    //
    // 這裡 `task("C", 1000)` 捕獲的是字串字面常數 `'static &str`，
    // 是 `Send` 的，所以可以安全地 spawn。切勿 spawn 捕獲非 'static 借用的 future。
    let start = Instant::now();
    let h1 = tokio::spawn(task("C", 1000));
    let h2 = tokio::spawn(task("D", 1000));
    // 等待兩個 task 都完成；`.await` 取回 `JoinHandle` 的結果。
    // `unwrap()` 是因為 `JoinHandle::await` 回傳 `Result<T, JoinError>`，
    // 當 task 內 panic 時會是 Err，這裡簡化處理直接 unwrap。
    h1.await.unwrap();
    h2.await.unwrap();
    println!("並發耗時: {:?}", start.elapsed());

    println!("---");

    println!("比較: 循序 ~2s vs 並發 ~1s");
}
