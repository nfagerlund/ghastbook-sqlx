# to actually run this thing

- `cargo install sqlx-cli`
- `sqlx db setup`
- `cargo run`

# Notes to self

## sqlx-cli

<https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md>

- It needs a `$DATABASE_URL`. Can take one from env var or a .env file.
    - the `sqlx::query!` macros require this also.
    - `sqlite:filename.db` (hmm)
- `db create` just creates
- `db setup` creates AND runs migrations
- `migrate run` runs migrations
- `migrate add NAME` makes a new sql migration file
    - OH!! Specify `-r` to make reversible migrations!!! neat. But, I guess you have to specify that when creating the FIRSt migration. Hmmm.
- `migrate info` lists em
- `migrate revert` ...I guess creates a new inverse migration? Or, what? huh.

### migrations in-app

> Did you know you can embed your migrations in your application binary?
> On startup, after creating your database connection or pool, add:
>
> sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?;
>
> Note that the compiler won't pick up new migrations if no Rust source files > have changed.
> You can create a Cargo build script to work around this with `sqlx migrate > build-script`.
>
> See: https://docs.rs/sqlx/0.5/sqlx/macro.migrate.html

a nice message I got on the cli just now.

### migrations in schema

after running db setup with one migration:

```sql
sqlite> .schema
CREATE TABLE _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);
CREATE TABLE visits(
    visitor TEXT PRIMARY KEY,
    count INTEGER
);
```

```
.headers ON
select * from _sqlx_migrations;
version|description|installed_on|success|checksum|execution_time
20240203024053|creation|2024-02-03 02:43:46|1|<BLOB GARBAGE>|404083
```

By the way, you can deal with the garbled output of BLOB typed columns by selecting columns individually and selecting `QUOTE(column)` for the affected one.

## sqlite itself

### sql parameters/placeholders

- [sqlite parameter binding docs](https://sqlite.org/c3ref/bind_blob.html)
- [sqlite language/expression docs on parameters](https://sqlite.org/lang_expr.html#varparam)

ok so in eardogger, my postgresql statements all used placeholders like `$1`, `$2` to bind to specific indices in the passed parameter list. And because this was JavaScript, it was easy to have an array of heterogeneous types.

In sqlite, it looks like our options are

- `?` -- auto-incrementing positional placeholder. this is what I used in the rusqlite ghastbook, due to all the examples. Obviously this sucks ass.
- `?1`, `?2`... -- manual positional placeholder. ok, word. Note that the count starts at 1 not 0.
- `:name`, `@name`, `$name` -- named parameters. If you're doing the C api yourself, you actually need to call a function to look up the underlying auto-incremented index of each named parameter and then pass the values by index. But wrapper libraries handle that for you.
    - The `$` version has something funny with `::` namespace separators and parentheses that I don't understand from the description.
    - The name of the parameter isn't `name`, but `:name` or `@name` etc.

#### Oh hell naw

sqlx does not support named query parameters in any way shape or form.

- <https://github.com/launchbadge/sqlx/issues/875>
- <https://github.com/launchbadge/sqlx/issues/199>

well, let's see if explicit positional works.

### string literals

Single-quotes. Postgres was like this too.

Apparently you encode a literal single quote with two single quotes in a row. hate it!!

## Time

what even is it, anyway.

std::time only does Instant and Duration, basically. It doesn't seem to know about clocks or calendars.

Both these crates seem actively maintained: (Old materials accuse both of them of abandonment at various times.)

- [chrono](https://github.com/chronotope/chrono)
- [time](https://github.com/time-rs/time)

It looks like chrono is more rigorous about separating timezone-aware types from naïve types.

## tracing and subscribing

this tracing stuff... woof... I guess it's complicated bc it's not just a logger, it's also what fuels like the Tracy integration in bevy, so it's got to be flexible and fast, plus there's all the span and scope stuff to associate events with logic zones in a way that chronological logs can't. anyway, it looks like:

- registry() makes a blank-ish subscriber that you can insert a bunch of layers into
- .with() calls insert layers that can filter events or choose to emit them to some form of output
- .init() makes the finished subscriber the default subscriber.
- the EnvFilter thing lets you set a string like the one you see inline via the $RUST_LOG env var.
    - for some reason, my crate name gets transformed to have an underscore instead of a hyphen, not sure what the rules for that are, maybe it's in the info!() macro somewhere? Anyway,
