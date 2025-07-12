mod order_book;
mod market;

use crate::market::MarketSimulator;
use crate::order_book::OrderSide;
use std::io::{self, Write};

fn main() {
    println!("=== Market Simulator ===");
    println!("Minimum spread: 1%");
    println!("Commands:");
    println!("  buy <price> <quantity>  - Place buy limit order");
    println!("  sell <price> <quantity> - Place sell limit order");
    println!("  cancel <order_id>       - Cancel order");
    println!("  status                  - Show market status");
    println!("  quit                    - Exit");
    println!();

    let mut market = MarketSimulator::new(1.0); // 1% minimum spread

    // Add some initial orders to demonstrate the market
    println!("Adding some initial orders...");
    
    // Initial buy orders
    if let Ok(order_id) = market.place_order(OrderSide::Buy, 100.0, 10) {
        println!("Placed initial buy order: {} shares at $100.00 (ID: {})", 10, order_id);
    }
    if let Ok(order_id) = market.place_order(OrderSide::Buy, 99.0, 15) {
        println!("Placed initial buy order: {} shares at $99.00 (ID: {})", 15, order_id);
    }

    // Initial sell orders (must be at least 1% spread from buy orders)
    if let Ok(order_id) = market.place_order(OrderSide::Sell, 102.0, 8) {
        println!("Placed initial sell order: {} shares at $102.00 (ID: {})", 8, order_id);
    }
    if let Ok(order_id) = market.place_order(OrderSide::Sell, 103.0, 12) {
        println!("Placed initial sell order: {} shares at $103.00 (ID: {})", 12, order_id);
    }

    market.print_market_status();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        
        match parts.get(0) {
            Some(&"quit") | Some(&"exit") => {
                println!("Goodbye!");
                break;
            }
            Some(&"status") => {
                market.print_market_status();
            }
            Some(&"buy") => {
                if parts.len() != 3 {
                    println!("Usage: buy <price> <quantity>");
                    continue;
                }
                
                match (parts[1].parse::<f64>(), parts[2].parse::<u64>()) {
                    (Ok(price), Ok(quantity)) => {
                        match market.place_order(OrderSide::Buy, price, quantity) {
                            Ok(order_id) => {
                                println!("Buy order placed: {} shares at ${:.2} (ID: {})", quantity, price, order_id);
                                
                                // Show any trades that occurred
                                let recent_trades = market.get_recent_trades(1);
                                if !recent_trades.is_empty() {
                                    let trade = recent_trades[0];
                                    println!("🔥 TRADE EXECUTED: {} shares at ${:.2}", trade.quantity, trade.price.into_inner());
                                }
                            }
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    _ => println!("Invalid price or quantity"),
                }
            }
            Some(&"sell") => {
                if parts.len() != 3 {
                    println!("Usage: sell <price> <quantity>");
                    continue;
                }
                
                match (parts[1].parse::<f64>(), parts[2].parse::<u64>()) {
                    (Ok(price), Ok(quantity)) => {
                        match market.place_order(OrderSide::Sell, price, quantity) {
                            Ok(order_id) => {
                                println!("Sell order placed: {} shares at ${:.2} (ID: {})", quantity, price, order_id);
                                
                                // Show any trades that occurred
                                let recent_trades = market.get_recent_trades(1);
                                if !recent_trades.is_empty() {
                                    let trade = recent_trades[0];
                                    println!("🔥 TRADE EXECUTED: {} shares at ${:.2}", trade.quantity, trade.price.into_inner());
                                }
                            }
                            Err(msg) => println!("Error: {}", msg),
                        }
                    }
                    _ => println!("Invalid price or quantity"),
                }
            }
            Some(&"cancel") => {
                if parts.len() != 2 {
                    println!("Usage: cancel <order_id>");
                    continue;
                }
                
                match parts[1].parse::<uuid::Uuid>() {
                    Ok(order_id) => {
                        match market.cancel_order(order_id) {
                            Some(order) => {
                                println!("Cancelled order: {} {} {} shares at ${:.2}", 
                                    match order.side {
                                        OrderSide::Buy => "BUY",
                                        OrderSide::Sell => "SELL",
                                    },
                                    order.remaining_quantity(),
                                    order.quantity,
                                    order.price.into_inner()
                                );
                            }
                            None => println!("Order not found"),
                        }
                    }
                    Err(_) => println!("Invalid order ID format"),
                }
            }
            Some(&"help") => {
                println!("Commands:");
                println!("  buy <price> <quantity>  - Place buy limit order");
                println!("  sell <price> <quantity> - Place sell limit order");
                println!("  cancel <order_id>       - Cancel order");
                println!("  status                  - Show market status");
                println!("  quit                    - Exit");
            }
            _ => {
                println!("Unknown command. Type 'help' for available commands.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_order_placement() {
        let mut market = MarketSimulator::new(1.0);
        
        // Place a buy order
        let buy_order_id = market.place_order(OrderSide::Buy, 100.0, 10).unwrap();
        assert!(market.get_order(&buy_order_id).is_some());
        
        // Place a sell order with sufficient spread
        let sell_order_id = market.place_order(OrderSide::Sell, 102.0, 5).unwrap();
        assert!(market.get_order(&sell_order_id).is_some());
    }

    #[test]
    fn test_minimum_spread_enforcement() {
        let mut market = MarketSimulator::new(1.0);
        
        // Place a buy order
        market.place_order(OrderSide::Buy, 100.0, 10).unwrap();
        
        // Try to place a sell order with insufficient spread (should fail)
        let result = market.place_order(OrderSide::Sell, 100.5, 5);
        assert!(result.is_err());
        
        // Place a sell order with sufficient spread (should succeed)
        let result = market.place_order(OrderSide::Sell, 102.0, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trade_execution() {
        let mut market = MarketSimulator::new(1.0);
        
        // Place a sell order first
        market.place_order(OrderSide::Sell, 100.0, 10).unwrap();
        
        // Place a buy order that crosses the spread and should execute immediately
        // We'll place it at the same price or higher to trigger execution
        market.place_order(OrderSide::Buy, 100.0, 5).unwrap();
        
        // Check that a trade occurred
        let trades = market.get_all_trades();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[0].price.into_inner(), 100.0); // Trade executes at the sell order's price
    }

    #[test]
    fn test_order_cancellation() {
        let mut market = MarketSimulator::new(1.0);
        
        let order_id = market.place_order(OrderSide::Buy, 100.0, 10).unwrap();
        
        let cancelled_order = market.cancel_order(order_id);
        assert!(cancelled_order.is_some());
        
        // Order should no longer be in the book
        let order = market.get_order(&order_id);
        assert!(order.is_none());
    }
}
