// 第 6 章：Web 開發（axum）
// 壓軸章：整合 async (tokio) + serde (JSON) + REST API (axum)
//
// 這是一個 Todo REST API，使用記憶體儲存（程式結束即消失）。
// 展示 axum 0.8 的 Router、Extractor、State、以及 Arc<Mutex<T>> 共用可變狀態。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// === 資料結構 ===

/// 一筆待辦事項。
/// `Serialize`/`Deserialize` 讓 serde 能自動把 struct 轉成 JSON（回應）
/// 與把 JSON 轉成 struct（請求 body）。
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

/// 建立待辦時的請求 body，只需 title（id 與 done 由伺服器產生）。
#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

/// 更新待辦時的請求 body，title 與 done 皆為可選（Option = 可能不存在於 JSON）。
/// 對應 Python：Pydantic 的 `Optional[str] = None`。
#[derive(Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    done: Option<bool>,
}

/// 應用程式共用狀態。
/// `Arc<Mutex<T>>`：Arc 讓多個 task 安全共用同一份資料（原子計數的指標），
/// Mutex 確保同一時間只有一個 task 能修改它。
/// 對應 Python：asyncio 中通常用 `asyncio.Lock` 包住共用 list/dict。
#[derive(Clone)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
    next_id: Arc<Mutex<u32>>,
}

// === Handler 函數 ===

/// GET /todos — 列出所有待辦。
/// `State(s): State<AppState>` 是 extractor：axum 從請求中取出注入的 state。
async fn list_todos(State(s): State<AppState>) -> Json<Vec<Todo>> {
    let todos = s.todos.lock().await;
    // 回傳 vec 的複本（clone），讓 lock 盡早釋放。
    Json(todos.clone())
}

/// POST /todos — 建立新待辦。
/// `Json(body): Json<CreateTodo>` 是 extractor：axum 自動把 request body
/// 的 JSON 反序列化成 `CreateTodo`（靠 serde）。對應 Python 的 Pydantic model。
async fn create_todo(
    State(s): State<AppState>,
    Json(body): Json<CreateTodo>,
) -> Json<Todo> {
    // 鎖住 next_id，取號後遞增
    let mut next_id = s.next_id.lock().await;
    let id = *next_id;
    *next_id += 1;
    drop(next_id); // 及早釋放 next_id 的 lock

    let todo = Todo {
        id,
        title: body.title,
        done: false,
    };

    let mut todos = s.todos.lock().await;
    todos.push(todo.clone());

    // Json(todo) 會自動把 struct 序列化成 JSON 並設定 Content-Type
    Json(todo)
}

/// GET /todos/{id} — 取得單筆待辦。
/// `Path(id): Path<u32>` 從 URL 路徑參數取出 id。
///
/// 回傳 `Result<Json<Todo>, (StatusCode, &'static str)>`：
/// Ok 回傳 JSON，Err 回傳狀態碼 + 錯誤訊息。
/// 這種寫法讓 get/update/delete 三個 handler 的回傳型別一致，方便維護。
async fn get_todo(
    State(s): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<Todo>, (StatusCode, &'static str)> {
    let todos = s.todos.lock().await;
    let todo = todos.iter().find(|t| t.id == id);

    match todo {
        Some(t) => Ok(Json(t.clone())),
        None => Err((StatusCode::NOT_FOUND, "找不到該待辦")),
    }
}

/// PUT /todos/{id} — 更新待辦（可改 title 與/或 done）。
/// `UpdateTodo` 的欄位是 `Option`，只有 Some 時才更新，None（或 JSON 缺該欄位）則保留原值。
async fn update_todo(
    State(s): State<AppState>,
    Path(id): Path<u32>,
    Json(body): Json<UpdateTodo>,
) -> Result<Json<Todo>, (StatusCode, &'static str)> {
    let mut todos = s.todos.lock().await;
    let todo = todos.iter_mut().find(|t| t.id == id);

    match todo {
        Some(t) => {
            // 只更新 Some 的欄位
            if let Some(title) = body.title {
                t.title = title;
            }
            if let Some(done) = body.done {
                t.done = done;
            }
            Ok(Json(t.clone()))
        }
        None => Err((StatusCode::NOT_FOUND, "找不到該待辦")),
    }
}

/// DELETE /todos/{id} — 刪除待辦。
/// 回傳 `StatusCode`（204 No Content 表示成功且無回應 body）。
async fn delete_todo(
    State(s): State<AppState>,
    Path(id): Path<u32>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let mut todos = s.todos.lock().await;
    // 找到 index 才能移除（Vec::retain 會過濾掉符合條件的元素）
    let before = todos.len();
    todos.retain(|t| t.id != id);

    if todos.len() < before {
        // 有刪除成功 -> 204 No Content
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "找不到該待辦"))
    }
}

// === 主函數 ===

// Python 開發者注意：
// 不像 Flask/FastAPI 路由參數直接出現在函數參數（如 `def user(id): ...`），
// axum 必須用 `Path(id): Path<u32>` extractor 從 URL 取出參數。
// State 必須實作 Clone（所以 AppState derive Clone）。
// 共用可變狀態用 `Arc<Mutex<T>>`：Arc 負責跨 task 共用，Mutex 負責安全修改。
// 另外注意：axum 0.8 的路徑參數語法是 `{id}`（大括號），舊版 `:id` 會在註冊路由時 panic。
#[tokio::main]
async fn main() {
    // 建立初始狀態：空的待辦清單，下一個 id 從 1 開始
    let state = AppState {
        todos: Arc::new(Mutex::new(Vec::new())),
        next_id: Arc::new(Mutex::new(1)),
    };

    // Router：把路徑對應到 handler。
    // `.route(path, method(handler))` 註冊一條路由；
    // 同一路徑可鏈接多個方法：`.get(...).post(...)`。
    //
    // axum 0.8 路徑語法：`{id}`（大括號），不是舊版的 `:id`。
    let app = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/{id}", get(get_todo).put(update_todo).delete(delete_todo))
        .with_state(state);

    // 綁定 TCP listener。0.0.0.0 表示監聽所有網卡。
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("伺服器啟動: http://localhost:3000");
    println!("試試看：");
    println!("  curl http://localhost:3000/todos");
    println!("  curl -X POST http://localhost:3000/todos -H 'Content-Type: application/json' -d '{{\"title\":\"學 Rust\"}}'");

    // axum::serve 是 0.7+ 的伺服器入口，
    // 接收一個 listener 與 app，跑直到關閉。
    axum::serve(listener, app).await.unwrap();
}
