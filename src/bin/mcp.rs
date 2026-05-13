use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use std::cmp::Ordering;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct AddParams {
    a: i32,
    b: i32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CompareParams {
    a: i32,
    b: i32,
}

#[derive(Debug, Clone, Default)]
struct Calculator;

#[tool_router(server_handler)]
impl Calculator {
    #[tool(description = "相加两个数值", title = "数值相加tool")]
    async fn add(&self, Parameters(AddParams { a, b }): Parameters<AddParams>) -> String {
        (a + b).to_string()
    }

    #[tool(description = "比较两个数值的大小", title = "比较数值tool")]
    async fn compare(
        &self,
        Parameters(CompareParams { a, b }): Parameters<CompareParams>,
    ) -> String {
        match a.cmp(&b) {
            Ordering::Greater => "1".into(),
            Ordering::Equal => "0".into(),
            Ordering::Less => "-1".into(),
        }
    }
}

const BIND_ADDRESS: &str = "localhost:8001";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let ct = tokio_util::sync::CancellationToken::new();
    let service = StreamableHttpService::new(
        || Ok(Calculator),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(ct.child_token()),
    );
    let cors = CorsLayer::new()
        // 允许本地常见 origin，也可以用 Origin::list 精确控制
        .allow_origin(AllowOrigin::any())
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([axum::http::HeaderName::from_static("mcp-session-id")]);
    let router = axum::Router::new()
        .route_service("/mcp", service)
        .layer(cors);
    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.unwrap();
            ct.cancel();
        })
        .await;
    Ok(())
}
