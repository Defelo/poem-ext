use poem::{
    http::{header, StatusCode},
    Request,
};
use poem_ext::{custom_auth, response};
use poem_openapi::{auth::Bearer, payload::PlainText, OpenApi};

use common::{get_client, get_spec};

mod common;

#[tokio::test]
async fn test_no_token() {
    let cli = get_client(Api);
    let resp = cli.get("/secret").send().await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_token() {
    let cli = get_client(Api);
    let resp = cli
        .get("/secret")
        .header(header::AUTHORIZATION, "Bearer invalid-token")
        .send()
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_forbidden() {
    let cli = get_client(Api);
    let resp = cli
        .get("/secret")
        .header(header::AUTHORIZATION, "Bearer user-token")
        .send()
        .await;
    resp.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_ok() {
    let cli = get_client(Api);
    let resp = cli
        .get("/secret")
        .header(header::AUTHORIZATION, "Bearer admin-token")
        .send()
        .await;
    resp.assert_status_is_ok();
    resp.assert_text("Logged in as admin (id=42)").await;
}

#[tokio::test]
async fn test_spec() {
    let cli = get_client(Api);
    let spec = get_spec(&cli).await;
    let spec = spec.value();

    let auth_scheme = spec
        .object()
        .get("components")
        .object()
        .get("securitySchemes")
        .object()
        .get("UserAuth")
        .object();
    auth_scheme.get("scheme").assert_string("bearer");
    auth_scheme.get("type").assert_string("http");
}

struct User {
    id: i32,
    name: String,
}

struct UserAuth(User);

response!(AuthResult = {
    Unauthorized(401, error),
    Forbidden(403, error),
});

async fn user_auth_check(
    _req: &Request,
    token: Option<Bearer>,
) -> Result<User, AuthResult::raw::Response> {
    let Bearer { token } = token.ok_or_else(AuthResult::raw::unauthorized)?;
    match token.as_str() {
        "admin-token" => Ok(User {
            id: 42,
            name: "admin".into(),
        }),
        "user-token" => Err(AuthResult::raw::forbidden()),
        _ => Err(AuthResult::raw::unauthorized()),
    }
}

custom_auth!(UserAuth, user_auth_check);

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/secret", method = "get")]
    async fn secret(&self, auth: UserAuth) -> PlainText<String> {
        let UserAuth(User { id, name }) = auth;
        PlainText(format!("Logged in as {name} (id={id})"))
    }
}
