use axum::{extract::Query, response::IntoResponse};

pub async fn serve_login_template() -> axum::response::Response {
    let file = std::fs::File::open("service-demo/src/auth/templates/login_with.html").unwrap();
    let contents = std::io::read_to_string(file).unwrap();
    let response = axum::response::Response::builder()
        .header("Content-Type", "text/html")
        .body(contents)
        .unwrap()
        .into_response();
    response
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginWithParams {
    pub tp: String,
}

pub async fn login_with(Query(params): Query<LoginWithParams>) -> axum::response::Response {
    let response = axum::response::Response::builder()
        .header("Content-Type", "text/html")
        .body(format!("Login with {} selected", params.tp))
        .unwrap()
        .into_response();
    response
}
