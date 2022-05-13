# Pocket Proxy server (Rust version)

This is an reimplementation of the [original Pocket proxy server](https://github.com/Pocket/proxy-server) in Rust. The proxy server delivers sponsored content obtained from the Kevel API (formerly Adzerk), while preserving the privacy of Firefox users.

The code of https://github.com/mozilla/classify-client was used as a starting point of this implementation.

## Dev instructions

This is a normal Cargo project, so after cloning the repository, you can build and run it with

```shell
$ cargo build
$ cargo run
```

This project should run on the latest stable version of Rust. Unstable features are not allowed.

### GeoIP Database

A GeoIP database will need to be provided. By default it is expected to be found at `./GeoIP2-City.mmdb`.

## Configuration

Via environment variables:

- `DEBUG`: Set to `"true"` to enable extra debugging options, such as a `/debug`
    endpoint that shows internal server state (default: `"false"`).
- `GEOIP_DB_PATH`: path to GeoIP database (default: `"./GeoIP2-City.mmdb"`)
- `HOST`: host to bind to (default: `"localhost"`)
- `HUMAN_LOGS`: set to `"true"` to use human readable logging (default: MozLog as JSON)
- `METRICS_TARGET`: The host and port to send statsd metrics to. May be a
    hostname like `"metrics.example.com:8125"` or an IP like
    `"127.0.0.1:8125"`. Port is required. (default: `"localhost:8125"`)
- `PORT`: port number to bind to (default: `"8000"`)
- `SENTRY_DSN`: report errors to a Sentry instance (default: `""`)
- `TRUSTED_PROXY_LIST`: A comma-separated list of CIDR ranges that trusted
    proxies will be in. Supports both IPv4 and IPv6.
- `VERSION_FILE`: path to `version.json` file (default: `"./version.json"`)

## Tests

Tests can be run with Cargo as well

```shell
$ cargo test
```

## Linting

Linting is handled via
[Therapist](https://therapist.readthedocs.io/en/latest/). After installing it,
enable the git hooks using either `therapist install` or `therapist install
--fix`. The `--fix` variant will automatically format your code upon commit.
The variant without `--fix` will simply show an error and ask you to reformat
the code using other means before committing.  Therapist runs in CI.

The checks Therapist runs are:

* Rustfmt
* Clippy, using the `clippy::all` preset
