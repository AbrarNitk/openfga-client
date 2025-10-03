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
    // redirect to dex idp for different providers
    // dex is running on port 5556
    // so we need to redirect to http://localhost:5556/auth/login-with?tp=google
    // or http://localhost:5556/auth/login-with?tp=github
    // or http://localhost:5556/auth/login-with?tp=facebook
    let redirect_url = format!("http://127.0.0.1:5556/auth/login-with?tp={}", params.tp);
    let response = axum::response::Response::builder()
        .header("Location", redirect_url)
        .status(axum::http::StatusCode::FOUND)
        .body(axum::body::Body::empty())
        .unwrap()
        .into_response();
    response
    // let response = axum::response::Response::builder()
    //     .header("Content-Type", "text/html")
    //     .body(format!("Login with {} selected", params.tp))
    //     .unwrap()
    //     .into_response();
    // response
}
