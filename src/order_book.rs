use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

pub type Price = OrderedFloat<f64>;
pub type Quantity = u64;
pub type OrderId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
    pub filled_quantity: Quantity,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
}

impl Order {
    pub fn new(side: OrderSide, price: f64, quantity: Quantity) -> Self {
        Self {
            id: Uuid::new_v4(),
            side,
            order_type: OrderType::Limit,
            price: OrderedFloat(price),
            quantity,
            filled_quantity: 0,
            status: OrderStatus::Open,
            timestamp: Utc::now(),
        }
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    pub fn is_complete(&self) -> bool {
        self.filled_quantity >= self.quantity
    }

    pub fn fill(&mut self, quantity: Quantity) {
        self.filled_quantity += quantity;
        if self.is_complete() {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub buy_order_id: OrderId,
    pub sell_order_id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: DateTime<Utc>,
}

impl Trade {
    pub fn new(buy_order_id: OrderId, sell_order_id: OrderId, price: Price, quantity: Quantity) -> Self {
        Self {
            id: Uuid::new_v4(),
            buy_order_id,
            sell_order_id,
            price,
            quantity,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    // Buy orders sorted by price (highest first), then by time (earliest first)
    buy_orders: BTreeMap<Price, Vec<Order>>,
    // Sell orders sorted by price (lowest first), then by time (earliest first)
    sell_orders: BTreeMap<Price, Vec<Order>>,
    // All orders by ID for quick lookup
    orders: HashMap<OrderId, Order>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            buy_orders: BTreeMap::new(),
            sell_orders: BTreeMap::new(),
            orders: HashMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        
        match order.side {
            OrderSide::Buy => {
                trades.extend(self.match_buy_order(order));
            }
            OrderSide::Sell => {
                trades.extend(self.match_sell_order(order));
            }
        }

        trades
    }

    fn match_buy_order(&mut self, mut buy_order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        // Try to match against existing sell orders
        let mut prices_to_remove = Vec::new();
        
        for (&sell_price, sell_orders) in self.sell_orders.iter_mut() {
            if buy_order.price < sell_price {
                break; // No more matches possible
            }

            let mut orders_to_remove = Vec::new();
            
            for (index, sell_order) in sell_orders.iter_mut().enumerate() {
                if buy_order.remaining_quantity() == 0 {
                    break;
                }

                let trade_quantity = buy_order.remaining_quantity().min(sell_order.remaining_quantity());
                let trade_price = sell_price; // Use the sell order's price

                // Create trade
                let trade = Trade::new(buy_order.id, sell_order.id, trade_price, trade_quantity);
                trades.push(trade);

                // Update orders
                buy_order.fill(trade_quantity);
                sell_order.fill(trade_quantity);

                // Update orders in the main HashMap
                self.orders.insert(buy_order.id, buy_order.clone());
                self.orders.insert(sell_order.id, sell_order.clone());

                if sell_order.is_complete() {
                    orders_to_remove.push(index);
                }
            }

            // Remove completed orders
            for &index in orders_to_remove.iter().rev() {
                sell_orders.remove(index);
            }

            if sell_orders.is_empty() {
                prices_to_remove.push(sell_price);
            }

            if buy_order.is_complete() {
                break;
            }
        }

        // Remove empty price levels
        for price in prices_to_remove {
            self.sell_orders.remove(&price);
        }

        // If buy order still has remaining quantity, add it to the book
        if !buy_order.is_complete() {
            self.orders.insert(buy_order.id, buy_order.clone());
            self.buy_orders
                .entry(buy_order.price)
                .or_insert_with(Vec::new)
                .push(buy_order);
        } else {
            self.orders.insert(buy_order.id, buy_order);
        }

        trades
    }

    fn match_sell_order(&mut self, mut sell_order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        // Try to match against existing buy orders (highest price first)
        let mut prices_to_remove = Vec::new();
        
        for (&buy_price, buy_orders) in self.buy_orders.iter_mut().rev() {
            if sell_order.price > buy_price {
                break; // No more matches possible
            }

            let mut orders_to_remove = Vec::new();
            
            for (index, buy_order) in buy_orders.iter_mut().enumerate() {
                if sell_order.remaining_quantity() == 0 {
                    break;
                }

                let trade_quantity = sell_order.remaining_quantity().min(buy_order.remaining_quantity());
                let trade_price = buy_price; // Use the buy order's price

                // Create trade
                let trade = Trade::new(buy_order.id, sell_order.id, trade_price, trade_quantity);
                trades.push(trade);

                // Update orders
                sell_order.fill(trade_quantity);
                buy_order.fill(trade_quantity);

                // Update orders in the main HashMap
                self.orders.insert(sell_order.id, sell_order.clone());
                self.orders.insert(buy_order.id, buy_order.clone());

                if buy_order.is_complete() {
                    orders_to_remove.push(index);
                }
            }

            // Remove completed orders
            for &index in orders_to_remove.iter().rev() {
                buy_orders.remove(index);
            }

            if buy_orders.is_empty() {
                prices_to_remove.push(buy_price);
            }

            if sell_order.is_complete() {
                break;
            }
        }

        // Remove empty price levels
        for price in prices_to_remove {
            self.buy_orders.remove(&price);
        }

        // If sell order still has remaining quantity, add it to the book
        if !sell_order.is_complete() {
            self.orders.insert(sell_order.id, sell_order.clone());
            self.sell_orders
                .entry(sell_order.price)
                .or_insert_with(Vec::new)
                .push(sell_order);
        } else {
            self.orders.insert(sell_order.id, sell_order);
        }

        trades
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Option<Order> {
        if let Some(mut order) = self.orders.remove(&order_id) {
            order.status = OrderStatus::Cancelled;
            
            // Remove from the appropriate side of the book
            match order.side {
                OrderSide::Buy => {
                    if let Some(orders) = self.buy_orders.get_mut(&order.price) {
                        orders.retain(|o| o.id != order_id);
                        if orders.is_empty() {
                            self.buy_orders.remove(&order.price);
                        }
                    }
                }
                OrderSide::Sell => {
                    if let Some(orders) = self.sell_orders.get_mut(&order.price) {
                        orders.retain(|o| o.id != order_id);
                        if orders.is_empty() {
                            self.sell_orders.remove(&order.price);
                        }
                    }
                }
            }
            
            Some(order)
        } else {
            None
        }
    }

    pub fn get_best_bid(&self) -> Option<Price> {
        self.buy_orders.keys().next_back().copied()
    }

    pub fn get_best_ask(&self) -> Option<Price> {
        self.sell_orders.keys().next().copied()
    }

    pub fn get_spread(&self) -> Option<f64> {
        match (self.get_best_bid(), self.get_best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.into_inner() - bid.into_inner()),
            _ => None,
        }
    }

    pub fn get_spread_percentage(&self) -> Option<f64> {
        match (self.get_best_bid(), self.get_best_ask()) {
            (Some(bid), Some(ask)) => {
                let mid_price = (bid.into_inner() + ask.into_inner()) / 2.0;
                let spread = ask.into_inner() - bid.into_inner();
                Some(spread / mid_price * 100.0)
            }
            _ => None,
        }
    }

    pub fn get_order(&self, order_id: &OrderId) -> Option<&Order> {
        self.orders.get(order_id)
    }

    pub fn get_market_depth(&self, levels: usize) -> (Vec<(Price, Quantity)>, Vec<(Price, Quantity)>) {
        let bids: Vec<(Price, Quantity)> = self.buy_orders
            .iter()
            .rev()
            .take(levels)
            .map(|(&price, orders)| {
                let total_quantity = orders.iter().map(|o| o.remaining_quantity()).sum();
                (price, total_quantity)
            })
            .collect();

        let asks: Vec<(Price, Quantity)> = self.sell_orders
            .iter()
            .take(levels)
            .map(|(&price, orders)| {
                let total_quantity = orders.iter().map(|o| o.remaining_quantity()).sum();
                (price, total_quantity)
            })
            .collect();

        (bids, asks)
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}
