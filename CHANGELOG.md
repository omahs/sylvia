# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.1](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.8.0...sylvia-derive-v0.8.1) - 2023-09-18

### Added
- Generate migrate entry point if message defined on contract

## [0.8.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.7.1...sylvia-derive-v0.8.0) - 2023-09-05

### Added
- Cast `deps` to empty
- Support QueryC associated type on interface
- Support custom queries on contracts

## [0.7.1](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.7.0...sylvia-derive-v0.7.1) - 2023-08-14

### Fixed
- Prefix interface proxy with module as Path

## [0.7.0](https://github.com/CosmWasm/sylvia/compare/sylvia-derive-v0.6.1...sylvia-derive-v0.7.0) - 2023-08-01

### Added
- Override generated entry_points
- Override entry_points in multitest helpers

### Fixed
- [**breaking**] `Remote` type implements all relevant traits so it can be stored in `#[cw_serde]` types

## [0.6.1] - 2023-06-28

- Fix dependencies in sylvia 0.6.0 (0.6.0 will be yanked)

## [0.6.0] - 2023-06-28

- InstantiateCtx and ReplyCtx are no longer type aliases (breaking)
- `multitest::App` is using more generic multitest version of `App`
- Support for custom messages via `#[sv::custom]` attribute

## [0.5.0] - 2023-05-26

- New `BoundQuerier` and `Remote` types are generated. Their goal is to make
  querying other contracts more intuitive.
- `module` attr for `contract` macro no longer wraps generated code in scope.
  As from now it will be used to provide path to contract implementation.
- Removed requirement for `const fn new()` method for `contract` macro call.
  `fn new()` method is still required.

## [0.4.2] - 2023-05-24

- Added support of `#[msg(reply)]` defining handler for reply messages,
  currently only in the form of
  `fn reply(&self, _ctx: ReplyCtx, _msg: Reply) -> Result<Response, Err>`
- Added generation of reply entrypoint forwarding to the `#[msg(reply)]`
  handler
- Added generation of reply implementation forwarding to `#[msg(reply)]`
  handler in multitest helpers

## [0.4.1] - 2023-05-23

- Lint fix

## [0.4.0] - 2023-05-16

- Introduced new `entry_points` macro
- Custom errors can be passed through `error` attribute

## [0.3.2] - 2023-04-18

- Changed the way multitest helpers are generated to avoid weird `use` statements in code.
- Introduced Context types in place of tuples
- Forwarding attributes on message fields
- Example usage of generated multitest helpers

## [0.3.1] - 2023-03-03

- Slight improvement the invalid message received error

## [0.3.0] - 2023-02-01

- Interfaces moved to separate directory to avoid errors on workspace optimizer
- `mt` feature added. Enabling it will:
  - generate `cw_multi_test::Contract` impl on a contract
  - generate Proxy to simplify writting tests
- Example of usage of new test framework
- Port of `cw20` contract on `sylvia` framework
- Default error type on contract is now `cosmwasm_std::StdError`
- Reexported `schemars`

## [0.2.2] - 2022-12-13

- Fix: Generate Migrate as struct
- Cw20 implementation in sylvia
- Removed `#[msg(reply)]`

## [0.2.1] - 2022-10-19

This is the first documented and supported implementation. It provides
macro to generate messsages for interfaces and contracts.

Some main points:

- Support for instantiate, execute, query, migrate and reply messages.
- Ability to implement multiple interfaces on contract.
- Mechanism of detecting overlapping of messages.
- Dispatch mechanism simplyfing entry points creation.
- Support for schema generation.
