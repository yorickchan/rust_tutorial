// 第 4 章：網路程式設計（reqwest）
//
// 用 reqwest 發送 HTTP 請求，用 serde 把 JSON 回應剖析成強型別的 struct。
// reqwest 的預設 Client 是非同步的（需要 Tokio runtime），所以 main 加上
// `#[tokio::main]` 並宣告為 `async fn`。

use serde::Deserialize;

// JSONPlaceholder 的 Todo 結構（camelCase 欄位）：
//   { "userId": 1, "id": 1, "title": "...", "completed": false }
//
// serde 預設用「欄位名稱完全相同」來對應。JSON 的 `userId` 是 camelCase，
// 而 Rust 慣用 snake_case（`user_id`），因此用 `#[serde(rename = "userId")]`
// 告訴 serde：「這個欄位在 JSON 裡叫做 userId」。
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Todo {
    #[serde(rename = "userId")]
    user_id: u32,
    id: u32,
    title: String,
    completed: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 建立一個可重用的 Client。它內部會維護 keep-alive 連線池（connection pool），
    // 對同一個 host 發多次請求時能重用連線，比每次都呼叫 `reqwest::get()`
    // （每次都建立一個新 Client）更有效率。
    let client = reqwest::Client::new();

    // === GET：抓取單一 Todo 並剖析成 struct ===
    // 1. client.get(url) 建立一個 GET 請求
    // 2. .send().await? 送出請求，拿到 Response
    // 3. .error_for_status()? 若回應狀態碼是 4xx/5xx，轉成 Err（否則回傳原 Response）
    let resp = client
        .get("https://jsonplaceholder.typicode.com/todos/1")
        .send()
        .await?
        .error_for_status()?;

    // resp.json::<T>() 把回應 body 當 JSON 剖析成型別 T。
    // 這裡 T = Todo，所以編譯期就知道結構（強型別）。
    // 注意：必須啟用 reqwest 的 `json` feature（見 Cargo.toml）。
    let todo: Todo = resp.json().await?;

    // {todo:#?} 用「美化（pretty-print）」的 Debug 格式印出整個 struct。
    println!("GET /todos/1:\n{todo:#?}");

    // === POST：送出 JSON body，伺服器會 echo 回來 ===
    // serde_json::json! 巨集讓我們用類似 JSON 的語法直接建立一個 serde_json::Value
    // （動態結構，類似 Python 的 dict）。這裡故意不另定義 Serialize struct，
    // 展示「動態 JSON」的寫法。
    let new_todo = serde_json::json!({
        "title": "學 Rust",
        "completed": false,
        "userId": 1,
    });

    // client.post(url).json(&value) 會自動：
    //   - 把 value 序列化成 JSON 放進 request body
    //   - 設定 Content-Type: application/json
    let resp = client
        .post("https://jsonplaceholder.typicode.com/todos")
        .json(&new_todo)
        .send()
        .await?
        .error_for_status()?;

    // 這次不剖析成特定 struct，而是用 serde_json::Value 接住任意 JSON。
    // JSONPlaceholder 的 POST 會回傳我們送出的內容（加上一個假的 id）。
    let body: serde_json::Value = resp.json().await?;
    println!("POST /todos (echo):\n{body:#?}");

    // === Python 開發者注意 ===
    // Python 的 `resp.json()` 回傳一個 dict，結構要到執行期才知道。
    // Rust 的 `resp.json::<Todo>().await?` 需在呼叫處指定型別 T，
    // 因此結構在「編譯期」就確定（強型別），欄位打錯或型別不符會直接編譯失敗。
    //
    // 此外，serde 預設會「忽略 JSON 中未知欄位」，
    // 不像 Pydantic 預設會對多餘欄位報錯。

    Ok(())
}
