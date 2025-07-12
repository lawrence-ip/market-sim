# Market Simulator Demo

This document shows example interactions with the market simulator.

## Starting the Simulator

```bash
cargo run
```

## Example Session

```
=== Market Simulator ===
Minimum spread: 1%

> status
=== MARKET STATUS ===
Best Bid: $100.00
Best Ask: $102.00
Spread: $2.00
Spread %: 1.98%

> buy 101.50 5
Buy order placed: 5 shares at $101.50 (ID: abc123...)

> sell 101.60 3
Sell order placed: 3 shares at $101.60 (ID: def456...)

> sell 100.00 2
🔥 TRADE EXECUTED: 2 shares at $100.00

> status
=== MARKET STATUS ===
Best Bid: $101.50
Best Ask: $101.60
Spread: $0.10
Spread %: 0.10%
```

## Key Features Demonstrated

1. **Minimum Spread Enforcement**: The simulator maintains a 1% minimum spread
2. **Order Matching**: Compatible orders are automatically matched
3. **Price-Time Priority**: Orders are processed in price-time priority
4. **Market Depth**: View current order book levels
5. **Trade Execution**: Real-time trade notifications

## Testing Edge Cases

Try these scenarios:
- Place orders that violate the minimum spread
- Cancel existing orders
- Place large orders that partially fill
- Monitor how the market depth changes with new orders
