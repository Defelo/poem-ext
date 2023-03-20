use poem::{
    http::StatusCode, middleware::CatchPanicEndpoint, test::TestClient, EndpointExt, Route,
};
use poem_ext::panic_handler::PanicHandler;
use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
use serde_json::json;

#[tokio::test]
async fn test_panic_handler() {
    let cli = get_client();
    let resp = cli.get("/test").send().await;
    resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    resp.assert_json(json!({"error": "internal_server_error"}))
        .await;
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/test", method = "get")]
    async fn test(&self) -> PlainText<&'static str> {
        panic!()
    }
}

fn get_client() -> TestClient<CatchPanicEndpoint<Route, PanicHandler>> {
    let api_service = OpenApiService::new(Api, "test", "test");
    let api = Route::new()
        .nest("/", api_service)
        .with(PanicHandler::middleware());
    TestClient::new(api)
}
