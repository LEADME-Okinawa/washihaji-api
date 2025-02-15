use std::{env, time::Duration};

use axum::{
    error_handling::HandleErrorLayer, extract::Query, http::StatusCode, response::IntoResponse,
    routing::get, Json, Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use dotenvy::dotenv;
use serde::Deserialize;
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, BoxError};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    let app = Router::new()
        .route("/", get(|| async { "home" }))
        .route("/api/v1/transform", get(rate_route))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
struct Rates {
    country: String,
    rate: BigDecimal,
}

#[derive(Debug, Default, Deserialize)]
struct Parameter {
    money: String,
    from: String,
    to: String,
}

use axum::debug_handler;

#[debug_handler]
async fn rate_route(Query(params): Query<Parameter>) -> impl IntoResponse {
    let db_url = env::var("DATABASE_URL").expect("db url is not valid!");
    let pool = sqlx::PgPool::connect(&db_url).await.unwrap();
    let data = rating(
        &pool,
        params.from,
        params.to,
        params.money.parse::<f32>().unwrap(),
    )
    .await;
    Json(data)
}

async fn rating(pool: &sqlx::PgPool, c01: String, c02: String, money: f32) -> f32 {
    let mut binding = get_rates(&pool, c01.clone(), c02).await.unwrap();
    let rate;
    let base = binding.pop().unwrap();
    if base.country != c01 {
        rate = BigDecimal::to_f32(&(base.rate.clone() / binding.last().unwrap().rate.clone()))
            .unwrap();
    } else {

        rate = BigDecimal::to_f32(&(binding.last().unwrap().rate.clone() / base.rate.clone()))
            .unwrap();
    }
    money * rate
}

async fn get_rates(
    pool: &sqlx::PgPool,
    c01: String,
    c02: String,
) -> Result<Vec<Rates>, sqlx::Error> {
    sqlx::query_as!(
        Rates,
        r#"
    SELECT country, rate
    FROM rates
    WHERE country = $1 OR country = $2
    "#,
        c01,
        c02
    )
    .fetch_all(pool)
    .await
}
