# Changelog

## [Unreleased]

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

[unreleased]: https://github.com/adriankumpf/alarmate/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/adriankumpf/alarmate/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/adriankumpf/alarmate/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/adriankumpf/alarmate/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/adriankumpf/alarmate/compare/cdb2267...v0.1.0
