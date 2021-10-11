[![release status](https://github.com/geoffjay/shook/actions/workflows/release.yml/badge.svg)](https://github.com/geoffjay/shook/actions?query=workflow%3A%22release%22)

# Shook

Trivial server to listen to Gitlab webhooks and execute a set of commands.

## Build

```shell
cargo build
```

## Execute

```shell
cargo run -- --token=super-gud-secret
```

To see all arguments that are available execute the command `cargo run -- --help`.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
