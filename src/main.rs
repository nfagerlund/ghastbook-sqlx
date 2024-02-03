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

// OK, baby's first tokio app...
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let pool = SqlitePool::connect("sqlite:./ghastbook.db").await?;
    let current = &args.visitor;

    // ok setup time's over baby
    // visit:
    visit(&pool, current, 1).await?;
    // dump:
    let results = fetch(&pool).await?;
    for Visitation { visitor, count } in results.iter() {
        println!("{}: {}", visitor, count);
    }

    Ok(())
}
