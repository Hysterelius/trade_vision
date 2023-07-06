
[![Crates.io](https://badgen.net/crates/v/trade_vision)](https://crates.io/crates/trade_vision)
[![Build Status](https://badgen.net/github/checks/hysterelius/trade_vision)](https://github.com/Hysterelius/trade_vision/actions)

#### _:rotating_light: trade_vision is currently in alpha, so it is not recommended for us in any production applications, as features will change rapidly._

# trade_vision

This is a pure Rust library which gets real time[^1] data from TradingView. Currently only supporting Rust, it can support JS/TS with Tauri and other equivalents.

[^1]: It can only retrieve the data from TradingView at default speeds, some markets may be time delayed.

This aims to be a rust equivalent of [TradingView-API](https://github.com/Mathieu2301/TradingView-API).

## Installation
```bash
cargo add trade_vision
```

or in your `cargo.toml`:
```toml
trade_vision = "0.1.0"
```

## Example uses
This library could be used for creating a stock trading bot or stock tracker.

Though it cannot currently retrieve graphs from TradingView.

Check out the examples folder for example uses!



## Features
- Realtime data[^1]


