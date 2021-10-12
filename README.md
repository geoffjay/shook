[![release status](https://github.com/geoffjay/shook/actions/workflows/release.yml/badge.svg)](https://github.com/geoffjay/shook/actions?query=workflow%3A%22release%22)

# Shook

Trivial server to listen to Gitlab webhooks and execute a set of commands.

## Install

```shell
curl -s https://api.github.com/repos/geoffjay/shook/releases/latest \
    | jq '.assets[] | select(.name|test("^shook.*linux-musl.zip$")) | .browser_download_url' \
    | tr -d \" \
    | wget -qi -
unzip $(find . -iname "shook_*.zip")
sudo mv shook /usr/local/bin/
```

Check the [setup](doc/SETUP.md) documentation for any remaining steps.

## Develop

### Build

```shell
cargo build
```

### Execute

```shell
cargo run
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
