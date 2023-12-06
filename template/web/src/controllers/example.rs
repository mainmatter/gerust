use axum::response::Html;

pub async fn hello() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
