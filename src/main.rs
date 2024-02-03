use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use clap::Parser;
use sqlx::{query, query_as, sqlite::SqlitePool, FromRow};

#[derive(Parser, Debug)]
struct Cli {
    visitor: String,
}

#[derive(Debug, FromRow)]
struct Visitation {
    visitor: String,
    count: i64,
}

// Reversing the params order to test explicit positional. fix later maybe.
const VISIT: &str = r#"
    INSERT INTO visits (visitor, count) VALUES (?2, ?1)
    ON CONFLICT(visitor) DO UPDATE
    SET count = count + ?1;
    "#;
const FETCH: &str = r#"
    SELECT visitor, count FROM visits;
    "#;

async fn visit(pool: &SqlitePool, visitor: &str, times: i64) -> anyhow::Result<()> {
    query(VISIT).bind(times).bind(visitor).execute(pool).await?;
    Ok(())
}

async fn fetch(pool: &SqlitePool) -> Result<Vec<Visitation>, sqlx::Error> {
    query_as::<_, Visitation>(FETCH).fetch_all(pool).await
}

async fn web_visit(
    Path(visitor): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<String, StatusCode> {
    // ok so this is basically our "main" now, logic wise

    let current = if visitor.is_empty() {
        "default buttmunch"
    } else {
        &visitor
    };
    // Do the visit, maybe barf.
    if (visit(&pool, current, 1).await).is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Get the leaderboard and return it
    match fetch(&pool).await {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(visits) => {
            let resp = visits
                .iter()
                .map(|v| format!("{}: {}\n", v.visitor, v.count))
                .fold(String::new(), |mut acc, line| {
                    acc.push_str(&line);
                    acc
                });
            Ok(resp)
        }
    }
}

// OK, baby's first tokio app...
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePool::connect("sqlite:./ghastbook.db").await?;

    let app = Router::new()
        .route("/:visitor", get(web_visit))
        .with_state(pool.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
