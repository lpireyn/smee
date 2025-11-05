# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0).

## [Unreleased]

### Added

- Added `hooks` subcommand to `smee` command
- Added `list` subcommand to `hooks` subcommand (#9)

### Changed

- Moved `activate` and `deactivate` subcommands under the new `hooks` subcommand

## [0.2.0] - 2025-10-16

### Added

- Added `activate` subcommand to `smee` command (#4)
- Added `deactivate` subcommand to `smee` command (#4)
- Added `--color` option to `smee` command (#1)
- Added support for the `SMEE_DISABLE` environment variable to disable the `hook` subcommand (#2)
- Added support for the `smee.maxLogLevel` Git configuration entry (#3)
- Added support for tilde expansion in `smee.dirs` Git configuration entry (#8)

## [0.1.0] - 2025-10-16

### Added

- Added `smee` command
- Added `install` subcommand to `smee` command
- Added `uninstall` subcommand to `smee` command
- Added `hook` subcommand to `smee` command
- Added `help` subcommand to `smee` command
- Added `--version` option to `smee` command
- Added `--help` option to `smee` command
- Added `--quiet` option to `smee` command
- Added `--verbose` option to `smee` command
