use crate::order_book::{Order, OrderBook, OrderSide, Trade, OrderId, Price, Quantity};

#[derive(Debug)]
pub struct MarketSimulator {
    order_book: OrderBook,
    trades: Vec<Trade>,
    minimum_spread_percentage: f64,
}

impl MarketSimulator {
    pub fn new(minimum_spread_percentage: f64) -> Self {
        Self {
            order_book: OrderBook::new(),
            trades: Vec::new(),
            minimum_spread_percentage,
        }
    }

    pub fn place_order(&mut self, side: OrderSide, price: f64, quantity: Quantity) -> Result<OrderId, String> {
        // Check minimum spread requirement
        if let Err(msg) = self.validate_order_spread(side, price) {
            return Err(msg);
        }

        let order = Order::new(side, price, quantity);
        let order_id = order.id;
        
        let new_trades = self.order_book.add_order(order);
        self.trades.extend(new_trades);
        
        Ok(order_id)
    }

    fn validate_order_spread(&self, side: OrderSide, price: f64) -> Result<(), String> {
        match side {
            OrderSide::Buy => {
                if let Some(best_ask) = self.order_book.get_best_ask() {
                    let ask_price = best_ask.into_inner();
                    // Allow buy orders at or above the best ask price (they will execute immediately)
                    if price >= ask_price {
                        return Ok(());
                    }
                    
                    let mid_price = (price + ask_price) / 2.0;
                    let spread_percentage = (ask_price - price) / mid_price * 100.0;
                    
                    if spread_percentage < self.minimum_spread_percentage {
                        return Err(format!(
                            "Buy order would create spread of {:.2}%, minimum required is {:.2}%",
                            spread_percentage, self.minimum_spread_percentage
                        ));
                    }
                }
            }
            OrderSide::Sell => {
                if let Some(best_bid) = self.order_book.get_best_bid() {
                    let bid_price = best_bid.into_inner();
                    // Allow sell orders at or below the best bid price (they will execute immediately)
                    if price <= bid_price {
                        return Ok(());
                    }
                    
                    let mid_price = (bid_price + price) / 2.0;
                    let spread_percentage = (price - bid_price) / mid_price * 100.0;
                    
                    if spread_percentage < self.minimum_spread_percentage {
                        return Err(format!(
                            "Sell order would create spread of {:.2}%, minimum required is {:.2}%",
                            spread_percentage, self.minimum_spread_percentage
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Option<Order> {
        self.order_book.cancel_order(order_id)
    }

    pub fn get_order(&self, order_id: &OrderId) -> Option<&Order> {
        self.order_book.get_order(order_id)
    }

    pub fn get_best_bid(&self) -> Option<Price> {
        self.order_book.get_best_bid()
    }

    pub fn get_best_ask(&self) -> Option<Price> {
        self.order_book.get_best_ask()
    }

    pub fn get_spread(&self) -> Option<f64> {
        self.order_book.get_spread()
    }

    pub fn get_spread_percentage(&self) -> Option<f64> {
        self.order_book.get_spread_percentage()
    }

    pub fn get_market_depth(&self, levels: usize) -> (Vec<(Price, Quantity)>, Vec<(Price, Quantity)>) {
        self.order_book.get_market_depth(levels)
    }

    pub fn get_recent_trades(&self, count: usize) -> Vec<&Trade> {
        self.trades.iter().rev().take(count).collect()
    }

    pub fn get_all_trades(&self) -> &Vec<Trade> {
        &self.trades
    }

    pub fn print_market_status(&self) {
        println!("\n=== MARKET STATUS ===");
        
        if let (Some(bid), Some(ask)) = (self.get_best_bid(), self.get_best_ask()) {
            println!("Best Bid: ${:.2}", bid.into_inner());
            println!("Best Ask: ${:.2}", ask.into_inner());
            
            if let Some(spread) = self.get_spread() {
                println!("Spread: ${:.2}", spread);
            }
            
            if let Some(spread_pct) = self.get_spread_percentage() {
                println!("Spread %: {:.2}%", spread_pct);
            }
        } else {
            println!("No active orders in the book");
        }

        let (bids, asks) = self.get_market_depth(5);
        
        println!("\nMarket Depth (Top 5 levels):");
        println!("BIDS\t\t\tASKS");
        println!("Price\tQuantity\tPrice\tQuantity");
        
        let max_levels = bids.len().max(asks.len());
        for i in 0..max_levels {
            let bid_str = if i < bids.len() {
                format!("{:.2}\t{}", bids[i].0.into_inner(), bids[i].1)
            } else {
                "\t".to_string()
            };
            
            let ask_str = if i < asks.len() {
                format!("{:.2}\t{}", asks[i].0.into_inner(), asks[i].1)
            } else {
                "".to_string()
            };
            
            println!("{}\t\t{}", bid_str, ask_str);
        }

        let recent_trades = self.get_recent_trades(3);
        if !recent_trades.is_empty() {
            println!("\nRecent Trades:");
            for trade in recent_trades {
                println!("Price: ${:.2}, Quantity: {}, Time: {}", 
                    trade.price.into_inner(), 
                    trade.quantity, 
                    trade.timestamp.format("%H:%M:%S"));
            }
        }
        
        println!("====================\n");
    }
}
