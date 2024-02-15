use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use clap::Parser;
use sqlx::{query, query_as, sqlite::SqlitePool, FromRow};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
struct Cli {
    /// Mandatory. The sqlite database URL to use, formatted like `sqlite:<PATH>`.
    #[arg(long, value_name = "URL")]
    db: Option<String>,
    /// Serve in FastCGI mode, for low-touch hosting with mod_fcgid. Conflicts with --port.
    #[arg(long)]
    fcgi: bool,
    /// The TCP port to serve the app on. Defaults to 3000. Conflicts with --fcgi.
    #[arg(long)]
    port: Option<u16>,
    // An alternate URI path to mount the app at, for shared domains. Use leading and
    // trailing slash, like `/nested/`.
    // #[arg(long, value_name = "PATH")]
    // mount: Option<String>,
}

#[derive(Debug, FromRow)]
struct Visitation {
    visitor: String,
    count: i64,
}

async fn visit(pool: &SqlitePool, visitor: &str, times: i64) -> anyhow::Result<()> {
    query!(
        r#"
        INSERT INTO visits (visitor, count) VALUES (?1, ?2)
        ON CONFLICT(visitor) DO UPDATE
        SET count = count + ?2;
        "#,
        visitor,
        times
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn fetch(pool: &SqlitePool) -> Result<Vec<Visitation>, sqlx::Error> {
    query_as!(
        Visitation,
        "SELECT visitor, count FROM visits ORDER BY count DESC;"
    )
    .fetch_all(pool)
    .await
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

async fn empty_web_visit(state: State<SqlitePool>) -> Result<String, StatusCode> {
    let path = Path("default lurnker".into());
    web_visit(path, state).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // validate and munge
    if args.fcgi && args.port.is_some() {
        panic!("The --fcgi and --port options are mutually exclusive. Choose one!");
    }
    // let mount = args.mount.as_deref().unwrap_or("/");
    let port = args.port.unwrap_or(3000);
    let Some(db_url) = args.db else {
        panic!("Must provide a database URL with --db");
    };

    // Set up debug logging for things that emit Tracing events
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ghastbook_sqlx=debug,sqlx=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = SqlitePool::connect(&db_url).await?;

    let app = Router::new()
        .route("/:visitor", get(web_visit))
        .route("/", get(empty_web_visit))
        .with_state(pool);

    if args.fcgi {
        busride_rs::serve_fcgid(app, 50.try_into()?).await?;
    } else {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .unwrap();
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, app).await?;
    }

    Ok(())
}
