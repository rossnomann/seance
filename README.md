# SEANCE

A session library.

[![CI](https://img.shields.io/github/actions/workflow/status/rossnomann/seance/ci.yml?style=flat-square)](https://github.com/rossnomann/seance/actions/)
[![Version](https://img.shields.io/crates/v/seance.svg?style=flat-square)](https://crates.io/crates/seance)
[![Downloads](https://img.shields.io/crates/d/seance.svg?style=flat-square)](https://crates.io/crates/seance)
[![Documentation](https://img.shields.io/badge/docs-latest-brightgreen.svg?style=flat-square)](https://docs.rs/seance)
[![License](https://img.shields.io/crates/l/seance.svg?style=flat-square)](https://github.com/rossnomann/seance/tree/0.19.0/LICENSE)

# Installation

```toml
[dependencies]
seance = "0.19.0"
```

# Example

See [tests](https://github.com/rossnomann/seance/tree/0.19.0/tests) directory.

# Changelog

## 0.19.0 (05.07.2025)

- Redis 0.32
- Set tokio version to 1.
- Set serde version to 1.
- Set serde_json version to 1.

## 0.18.0 (12.04.2025)

- Tokio 1.44
- Redis 0.29

## 0.17.0 (12.02.2025)

- Tokio 1.43
- Redis 0.28

## 0.16.0 (04.12.2024)

- Tokio 1.42

## 0.15.0 (01.11.2024)

- Tokio 1.41
- Redis 0.27

## 0.14.0 (07.09.2024)

- Tokio 1.40

## 0.13.0 (31.07.2024)

- Tokio 1.39
- Redis 0.26

## 0.12.0 (18.06.2024)

- Tokio 1.38

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
