#![allow(dead_code)]

use poem::{
    test::{TestClient, TestJson, TestJsonObject, TestJsonValue},
    Endpoint, EndpointExt, Route,
};
use poem_ext::panic_handler::PanicHandler;
use poem_openapi::{OpenApi, OpenApiService};

pub fn get_client(api: impl OpenApi + 'static) -> TestClient<impl Endpoint> {
    let api_service = OpenApiService::new(api, "test", "test");
    let api = Route::new()
        .nest("/openapi.json", api_service.spec_endpoint())
        .nest("/", api_service)
        .with(PanicHandler::middleware());
    TestClient::new(api)
}

pub async fn get_spec(client: &TestClient<impl Endpoint>) -> TestJson {
    let resp = client.get("/openapi.json").send().await;
    resp.assert_status_is_ok();
    resp.json().await
}

pub fn get_endpoint(
    spec: TestJsonValue,
    method: impl AsRef<str>,
    path: impl AsRef<str>,
) -> TestJsonObject {
    spec.object()
        .get("paths")
        .object()
        .get(path)
        .object()
        .get(method)
        .object()
}

pub fn check_description(
    spec: TestJsonValue,
    method: impl AsRef<str>,
    path: impl AsRef<str>,
    status: impl AsRef<str>,
    description: &str,
) {
    get_endpoint(spec, method, path)
        .get("responses")
        .object()
        .get(status)
        .object()
        .get("description")
        .assert_string(description);
}

pub fn check_schema(
    spec: TestJsonValue,
    method: impl AsRef<str>,
    path: impl AsRef<str>,
    status: impl AsRef<str>,
    ref_: &str,
) {
    get_endpoint(spec, method, path)
        .get("responses")
        .object()
        .get(status)
        .object()
        .get("content")
        .object()
        .get("application/json; charset=utf-8")
        .object()
        .get("schema")
        .object()
        .get("$ref")
        .assert_string(ref_);
}
