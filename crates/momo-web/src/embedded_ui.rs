use axum::response::Html;

pub async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

const INDEX_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/index.html"));
