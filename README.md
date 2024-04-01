# SEANCE

A session library written in Rust

[![CI](https://img.shields.io/github/actions/workflow/status/rossnomann/seance/ci.yml?style=flat-square)](https://github.com/rossnomann/seance/actions/)
[![Version](https://img.shields.io/crates/v/seance.svg?style=flat-square)](https://crates.io/crates/seance)
[![Downloads](https://img.shields.io/crates/d/seance.svg?style=flat-square)](https://crates.io/crates/seance)
[![Documentation](https://img.shields.io/badge/docs-latest-brightgreen.svg?style=flat-square)](https://docs.rs/seance)
[![License](https://img.shields.io/crates/l/seance.svg?style=flat-square)](https://github.com/rossnomann/seance/tree/0.11.0/LICENSE)

# Installation

```toml
[dependencies]
seance = "0.11.0"
```

# Example

See [tests](https://github.com/rossnomann/seance/tree/0.11.0/tests) directory.

# Changelog

## 0.11.0 (01.04.2024)

- Tokio 1.37
- Redis 0.25

## 0.10.0 (18.02.2024)

- Tokio 1.36

## 0.9.0 (01.01.2024)

- Tokio 1.35
- Redis 0.24
- Removed async-trait dependency.

## 0.8.0 (28.11.2023)

- Tokio 1.34
- Redis 0.23

## 0.7.0 (02.02.2022)

- Tokio 1.16.
- Removed snafu dependency.

## 0.6.0 (29.12.2021)

- Tokio 1.15.
- Redis 0.21.

## 0.5.0 (06.01.2020)

- Tokio 1.0 support.
- Use redis-rs for redis-backend instead of darkredis.
- Renamed RedisError to RedisBackendError.
- Renamed FilesystemError to FilesystemBackendError.

## 0.4.0 (09.03.2020)

- Added darkredis 0.7 support.

## 0.3.0 (23.02.2020)

- Added darkredis 0.6 support.

## 0.2.0 (04.01.2020)

- Send and Sync for SessionError.

## 0.1.0 (04.01.2020)

- First release.

# LICENSE

The MIT License (MIT)
