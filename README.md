# hinge-rs

[![Crates.io](https://img.shields.io/crates/v/hinge-rs)](https://crates.io/crates/hinge-rs)
[![docs.rs](https://docs.rs/hinge-rs/badge.svg)](https://docs.rs/hinge-rs)
[![CI](https://github.com/f0rr0/hinge-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/f0rr0/hinge-rs/actions/workflows/ci.yml)
[![API Reference](https://github.com/f0rr0/hinge-rs/actions/workflows/docs.yml/badge.svg)](https://f0rr0.github.io/hinge-rs/)
[![MSRV](https://img.shields.io/badge/msrv-1.88-blue)](https://www.rust-lang.org)
[![License](https://img.shields.io/crates/l/hinge-rs)](LICENSE-MIT)

Unofficial, typed Rust client for Hinge APIs, including Sendbird chat.

`hinge-rs` is built for correctness-first automation: typed REST calls, typed chat events, safe secret handling, raw escape hatches, and a generated Scalar API reference.

## Install

```bash
cargo add hinge-rs
```

## Use

```rust
use hinge_rs::Client;

#[tokio::main]
async fn main() -> Result<(), hinge_rs::errors::HingeError> {
    let mut client = Client::builder()
        .phone_number("+15555550123")
        .build()?;

    client.auth().initiate_sms().await?;
    client.auth().submit_otp("123456").await?;

    let recs = client.recommendations().get().await?;
    println!("{} feeds", recs.feeds.len());

    Ok(())
}
```

## Covers

- Hinge auth, profiles, recommendations, likes, ratings, prompts, connections, and settings.
- Sendbird channels, messages, and WebSocket events.
- OpenAPI JSON and Scalar docs generated from the crate.

## Docs

- Rust API docs: <https://docs.rs/hinge-rs>
- API reference: <https://f0rr0.github.io/hinge-rs/>
- Release snapshots use `/v/<version>/` paths.

## Status

Early OSS extraction. Unofficial and not affiliated with Hinge, Match Group, or Sendbird.
