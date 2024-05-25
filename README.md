## About the project
This thing is not to be touched under any circumstances.
It was made under duress for a not-so-good man.
It will only lie here so that sometime later, maybe (or maybe not), I will come back here to peek at a piece of something or scold myself.

## What I used
The backend was written in `Rust` using [Actix Web](https://actix.rs) to run the server itself, [Tera](https://keats.github.io/tera/) to generate html pages and responses, and [Tokio Postgres](https://docs.rs/tokio-postgres/latest/tokio_postgres/#) to access the Postgresql database (this thing is particularly badly done) (and I don't want to throw the database in here).
The pages are kept bare `HTML`, `CSS`, a bit of `JS`, messages are passed between client and server via `HTMX` (great stuff, used it with gusto)

## How to use
You need to make some exactly suitable database, create a user there with the name and password rust
To compile from the project folder write
```
cargo run
```
Go to `localhost:8080` and do not look further, if you still need eyesight
After the first compilation, the executable can be found at the path `target\debug\`, if compiled with the `--release` attribute, the path will be `target\release\`
