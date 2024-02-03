ok, so I'm doing a `cargo install sqlx-cli` help manage the migrations. this will come up later.

Notes to self:

## sqlx-cli

<https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md>

- It needs a `$DATABASE_URL`. Can take one from env var or a .env file.
    - the `sqlx::query!` macros require this also.
    - `sqlite:filename.db` (hmm)

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

### string literals

Single-quotes. Postgres was like this too.

Apparently you encode a literal single quote with two single quotes in a row. hate it!!
