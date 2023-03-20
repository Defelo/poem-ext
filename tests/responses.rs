#![allow(clippy::disallowed_names)]

use poem::http::StatusCode;
use poem_ext::{add_response_schemas, response};
use poem_openapi::{payload::Json, Object, OpenApi};
use serde_json::json;

use common::{check_description, check_schema, get_client, get_spec};

mod common;

#[tokio::test]
async fn test_no_auth() {
    let cli = get_client(Api);
    let resp = cli.get("/test").send().await;
    resp.assert_status_is_ok();
    resp.assert_json(json!({"foo": 42, "bar": "Hello World!"}))
        .await;
}

#[tokio::test]
async fn test_auth() {
    let cli = get_client(Api);
    let resp = cli
        .post("/test_auth")
        .body_json(&json!({"foo": 43, "bar": "test"}))
        .send()
        .await;
    resp.assert_status(StatusCode::CREATED);
    resp.assert_json(json!({})).await;

    let resp = cli
        .post("/test_auth")
        .body_json(&json!({"foo": 42, "bar": "test"}))
        .send()
        .await;
    resp.assert_status(StatusCode::CONFLICT);
    resp.assert_json(
        json!({"error": "conflict", "details": {"foo": 42, "other_bar": "Hello World!"}}),
    )
    .await;
}

#[tokio::test]
async fn test_spec() {
    let cli = get_client(Api);
    let spec = get_spec(&cli).await;
    let spec = spec.value();

    check_schema(spec, "get", "/test", "200", "#/components/schemas/Data");

    check_description(spec, "post", "/test_auth", "201", " data has been created");
    check_schema(
        spec,
        "post",
        "/test_auth",
        "201",
        "#/components/schemas/Empty",
    );
    check_description(
        spec,
        "post",
        "/test_auth",
        "409",
        " data conflicts with other data",
    );
    check_schema(
        spec,
        "post",
        "/test_auth",
        "409",
        "#/components/schemas/__TestAuth__Conflict",
    );
    check_description(
        spec,
        "post",
        "/test_auth",
        "401",
        " you need to authenticate",
    );
    check_schema(
        spec,
        "post",
        "/test_auth",
        "401",
        "#/components/schemas/__AuthResult__Unauthorized",
    );
    check_description(
        spec,
        "post",
        "/test_auth",
        "403",
        " you are not allowed to do this",
    );
    check_schema(
        spec,
        "post",
        "/test_auth",
        "403",
        "#/components/schemas/__AuthResult__Forbidden",
    );
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/test", method = "get")]
    async fn test(&self) -> Test::Response {
        Test::ok(Data {
            foo: 42,
            bar: "Hello World!".into(),
        })
    }

    #[oai(path = "/test_auth", method = "post")]
    async fn test_auth(&self, data: Json<Data>) -> TestAuth::Response<UserAuth> {
        match data.0.foo {
            42 => TestAuth::conflict(ConflictDetails {
                foo: 42,
                other_bar: "Hello World!".into(),
            }),
            _ => TestAuth::created(),
        }
    }
}

#[derive(Debug, Object)]
pub struct Data {
    foo: i32,
    bar: String,
}

response!(Test = {
    Ok(200) => Data,
});

response!(TestAuth = {
    /// data has been created
    Created(201),
    /// data conflicts with other data
    Conflict(409, error) => ConflictDetails,
});

#[derive(Debug, Object)]
pub struct ConflictDetails {
    foo: i32,
    other_bar: String,
}

struct UserAuth;
response!(AuthResult = {
    /// you need to authenticate
    Unauthorized(401, error),
    /// you are not allowed to do this
    Forbidden(403, error),
});
add_response_schemas!(UserAuth, AuthResult::raw::Response);
