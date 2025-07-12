# Market Simulator

A comprehensive market simulation program written in Rust that implements a limit order book with buy and sell orders, enforcing a minimum spread of 1%.

## Features

- **Limit Orders**: Support for buy and sell limit orders
- **Order Matching**: Automatic matching of compatible orders with price-time priority
- **Minimum Spread**: Enforces a 1% minimum spread between best bid and ask
- **Order Management**: Place, cancel, and track orders
- **Market Depth**: View market depth and order book levels
- **Trade History**: Track all executed trades
- **Real-time Status**: Monitor market status and recent activity

## Architecture

The simulator consists of three main components:

### Order Book (`order_book.rs`)
- Manages buy and sell orders in price-time priority queues
- Handles order matching and trade execution
- Provides market depth and spread calculations

### Market Simulator (`market.rs`)
- Orchestrates the order book operations
- Enforces minimum spread requirements
- Manages trade history and market statistics

### Interactive CLI (`main.rs`)
- Provides a command-line interface for market interaction
- Includes comprehensive testing suite

## Usage

### Building and Running

```bash
# Build the project
cargo build --release

# Run the simulator
cargo run

# Run tests
cargo test
```

### Commands

- `buy <price> <quantity>` - Place a buy limit order
- `sell <price> <quantity>` - Place a sell limit order
- `cancel <order_id>` - Cancel an existing order
- `status` - Display current market status
- `quit` - Exit the simulator

### Example Session

```
=== Market Simulator ===
Minimum spread: 1%

> buy 100.00 10
Buy order placed: 10 shares at $100.00 (ID: abc123...)

> sell 102.00 5
Sell order placed: 5 shares at $102.00 (ID: def456...)

> status
=== MARKET STATUS ===
Best Bid: $100.00
Best Ask: $102.00
Spread: $2.00
Spread %: 1.98%
...

> sell 100.00 3
🔥 TRADE EXECUTED: 3 shares at $100.00
```

## Key Features

### Minimum Spread Enforcement
The simulator enforces a 1% minimum spread between the best bid and ask prices. Orders that would violate this constraint are rejected with an appropriate error message.

### Price-Time Priority
Orders are matched based on price priority first, then time priority for orders at the same price level.

### Partial Fills
Orders can be partially filled if there isn't sufficient quantity available at the requested price level.

### Trade Execution
When orders match, trades are automatically executed and recorded with timestamps and unique IDs.

## Testing

The project includes comprehensive unit tests covering:
- Basic order placement and management
- Minimum spread enforcement
- Trade execution logic
- Order cancellation

Run tests with:
```bash
cargo test
```

## Dependencies

- `chrono` - Date and time handling
- `uuid` - Unique identifier generation
- `serde` - Serialization support
- `ordered-float` - Ordered floating-point numbers for price handling
