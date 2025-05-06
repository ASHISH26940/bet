use std::collections::HashMap;
use std::time::Instant;
use crate::crypto::crypto_to_usd;

#[derive(Default)]
pub struct Game {
    balances: HashMap<String, f64>,
    active_bets: HashMap<String, Bet>,
    crypto_prices: HashMap<String, f64>, // Store crypto prices like SOL and ETH
}

impl Game {
    // Register a new user and return the user_id
    pub fn register_user(&mut self) -> String {
        let user_id = format!("user_{}", self.balances.len() + 1);
        self.balances.insert(user_id.clone(), 1000.0); // Starting balance, example value
        user_id
    }

    // Place a bet with crypto and amount
    pub fn place_bet(&mut self, user_id: &str, bet: Bet) -> bool {
        let balance = self.balances.entry(user_id.to_string()).or_insert(0.0);
        
        // Check if user has enough balance
        if *balance >= bet.amount_crypto {
            // Deduct the bet amount from balance
            *balance -= bet.amount_crypto;
            
            // Store the active bet
            self.active_bets.insert(user_id.to_string(), bet);
            return true;
        }
        false
    }

    // Cash out and convert crypto to USD
    pub fn cash_out(&mut self, user_id: &str, multiplier: f64, price_per_crypto: f64) -> Option<f64> {
        // Retrieve and remove the active bet
        let bet = self.active_bets.remove(user_id)?;
        
        // Calculate winnings in crypto
        let winnings_crypto = bet.amount_crypto * multiplier;
        
        // Convert the crypto winnings to USD
        let winnings_usd = crypto_to_usd(winnings_crypto, price_per_crypto);
        
        // Ensure winnings are not negative
        let safe_winnings = winnings_usd.max(0.0);
        
        // Update the user's balance with USD equivalent
        let balance = self.balances.entry(user_id.to_string()).or_insert(0.0);
        *balance += safe_winnings;
        
        Some(*balance) // Return the new balance in USD
    }

    // Get user balance
    pub fn get_balance(&self, user_id: &str) -> Option<f64> {
        self.balances.get(user_id).copied()
    }

    // Set a crypto price (SOL, ETH, etc.)
    pub fn set_crypto_price(&mut self, crypto: &str, price: f64) {
        self.crypto_prices.insert(crypto.to_string(), price);
    }
    
    // Get current price of a crypto
    pub fn get_crypto_price(&self, crypto: &str) -> f64 {
        *self.crypto_prices.get(crypto).unwrap_or(&1.0)
    }
}

// Bet structure to store information about each bet
#[derive(Clone)]
pub struct Bet {
    pub crypto: String,
    pub amount_crypto: f64,
    pub start_time: Instant,
}