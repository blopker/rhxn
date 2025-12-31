mod api;
mod db;
mod templates;
mod types;

use askama::Template;
use axum::{Router, extract, http::StatusCode, response::IntoResponse, routing::get};
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with some routes
    let app = app();
    tokio::spawn(async {
        loop {
            if let Err(e) = api::run().await {
                eprintln!("API error: {}", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
        }
    });

    // run it
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();
    tracing::debug!("listening on http://{}", listener.local_addr().unwrap());
    let _ = axum::serve(listener, app).await;
}

fn app() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/item/{id}", get(item))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(CompressionLayer::new())
}

async fn item(extract::Path(id): extract::Path<String>) -> impl IntoResponse {
    let id = match id.parse::<types::ItemID>() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID").into_response(),
    };

    // Try cache first, otherwise fetch from API
    let item = match db::DB.items.get(&id) {
        Some(item) => Some(item),
        None => api::fetch_item_tree(id).await,
    };

    match item {
        Some(item) if matches!(item.item_type, Some(types::ItemType::Story)) => {
            let comments = render_comments(&item);
            let title = item.title.clone().unwrap_or_else(|| "Unknown".to_string());
            let template = templates::ItemTemplate {
                item,
                title,
                static_base: "/assets",
                comments,
            };
            templates::HtmlTemplate(template).into_response()
        }
        _ => (StatusCode::NOT_FOUND, "nothing to see here").into_response(),
    }
}

fn render_comments(item: &types::Item) -> Option<String> {
    // Collect only the comment IDs we need (BFS)
    let mut queue: Vec<types::ItemID> = item.kids.clone();
    let mut items = std::collections::HashMap::new();

    while let Some(id) = queue.pop() {
        if let Some(child) = db::DB.items.get(&id) {
            queue.extend(child.kids.iter().copied());
            items.insert(id, child);
        }
    }

    if items.is_empty() {
        return None;
    }

    let mut out = String::with_capacity(4096);
    render_comments_inner(item, &items, &mut out);
    if out.is_empty() { None } else { Some(out) }
}

fn render_comments_inner(
    item: &types::Item,
    items: &std::collections::HashMap<types::ItemID, types::Item>,
    out: &mut String,
) {
    for id in &item.kids {
        if let Some(comment) = items.get(id)
            && comment.by.is_some()
        {
            let mut child_html = String::new();
            render_comments_inner(comment, items, &mut child_html);
            out.push_str(
                &templates::CommentTemplate {
                    item: comment.clone(),
                    comments: if child_html.is_empty() { None } else { Some(child_html) },
                }
                .render()
                .unwrap(),
            );
        }
    }
}

async fn index() -> impl IntoResponse {
    let template = templates::IndexTemplate {
        stories: db::DB.get_top_stories(),
        title: "Stories",
        static_base: "/assets",
    };
    templates::HtmlTemplate(template)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use axum::{
//         body::Body,
//         http::{Request, StatusCode},
//     };
//     use http_body_util::BodyExt;
//     use tower::ServiceExt;

//     #[tokio::test]
//     async fn test_main() {
//         let response = app()
//             .oneshot(
//                 Request::builder()
//                     .uri("/greet/Foo")
//                     .body(Body::empty())
//                     .unwrap(),
//             )
//             .await
//             .unwrap();
//         assert_eq!(response.status(), StatusCode::OK);
//         let body = response.into_body();
//         let bytes = body.collect().await.unwrap().to_bytes();
//         let html = String::from_utf8(bytes.to_vec()).unwrap();

//         assert_eq!(html, "<h1>Hello, Foo!</h1>");
//     }
// }
