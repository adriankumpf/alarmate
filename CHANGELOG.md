# Changelog

## [Unreleased]

## [0.5.0] - 2026-08-10

### Breaking Changes

- `Client` methods take `&self` instead of `&mut self`, so one client can be
  shared across tasks behind an `Arc`
- `Error` and its `UnexpectedResponse` variant are now `#[non_exhaustive]`
- Rename `DeviceKind::ThermostatRcs_` to `ThermostatRcs`
- Drop the unused `clap::ValueEnum` derive from `Status`, `State` and
  `DeviceKind`
- `build-binary` no longer implies the `clap` and `tokio` optional-dependency
  features; depend on `build-binary` itself

### Added

- Export `State` and `Status`, which were reachable through `Device`'s public
  fields but could not be named
- Re-export `StatusCode`, so `Error::UnexpectedResponse` can be inspected
  without depending on `reqwest` directly
- `Modes` implements `Index<Area>`, so an area's mode can be looked up
  dynamically: `modes[Area::Area1]`
- `Client` implements `Debug`, redacting the credentials, so it can sit in a
  `Debug` application state
- `Clone`, `PartialEq` and `Eq` for `Device` and `Modes` (`Modes` is also `Copy`)
- Long forms for the CLI connection flags (`--ip-address`, `--username`,
  `--password`), which the README already documented
- `alarmate --version`, and the crate description in `--help`
- Crate metadata for publishing: `description`, `repository`, `license`,
  `keywords`, `categories` and a `rust-version` of 1.85

### Changed

- Retry on HTTP 401 Unauthorized errors in addition to session timeouts; the
  panel returns both transiently
- Treat a redirect to the panel's login page as a session timeout, which
  detects an expired session even when the login page itself does not mention
  the login path

### Fixed

- `--help` no longer prints the value of `ALARMATE_PASSWORD` in cleartext
- A session timeout during a GET now drops the cached token, instead of leaving
  a dead one behind for the next POST to fail on
- A session that expires while fetching a token is now retried; previously only
  the action request itself was, so the token fetch failed the whole call
- Fetching a token no longer nests one retry inside another, which let a single
  `change_mode` issue up to six requests
- Enum values now round-trip through serde: `Deserialize` accepts the variant
  name that `Serialize` writes, not just the discriminant
- The CLI reports errors via `Display` and exits non-zero, rather than printing
  the `Debug` representation

### Other Changes

- Add request and connect timeouts, and disable proxy auto-detection so
  credentials cannot be routed through a `HTTPS_PROXY`
- Bound the response body retained in `Error::UnexpectedResponse` to 512 bytes
- Only reparse a response body without tabs when it fails to parse with them
- The CLI uses a current-thread runtime rather than a worker pool
- Update dependencies

## [0.4.0] - 2026-02-22

### Breaking Changes

- Refactor `Client`: `new()` now returns `Result`, methods require `&mut self`, `change_mode()` takes `Mode` by value, `Client` no longer `Clone`
- Refactor error types: new `Unauthorized` variant, changed `Deserialize` variant, `is_session_timeout()` made private
- Make `Device` fields public

### Other Changes

- Upgrade reqwest to 0.13, switch to native-tls
- Replace `enum_number!` macro with strum/num_enum derives
- Improve handling of session timeouts
- Add CI workflow and tests
- Bump edition to 2024
- Update dependencies

## [0.3.0] - 2022-12-03

- Upgrade clap to v4
- Use async/await

## [0.2.0] - 2019-01-28

- `Client.get_status` returns distinct `Modes` struct
- Serialize `constants` as strings

## [0.1.0] - 2019-01-14

[unreleased]: https://github.com/adriankumpf/alarmate/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/adriankumpf/alarmate/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/adriankumpf/alarmate/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/adriankumpf/alarmate/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/adriankumpf/alarmate/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/adriankumpf/alarmate/compare/cdb2267...v0.1.0
