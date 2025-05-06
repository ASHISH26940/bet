use std::collections::HashMap;
use rand::Rng;

pub fn simulate_price(current_price: f64) -> f64 {
    let mut rng = rand::thread_rng();
    
    // Create more realistic market-like randomness
    
    // 1. Base volatility (normal market conditions: ±2%)
    let base_change = (rng.r#gen::<f64>() - 0.5) * 0.04;
    
    // 2. Occasional larger moves (mimicking sudden market shifts)
    let rare_event = if rng.r#gen::<f64>() < 0.1 {  // 10% chance of larger move
        let direction = if rng.r#gen::<bool>() { 1.0 } else { -1.0 };
        direction * rng.r#gen_range(0.03..0.08)  // 3-8% additional move
    } else {
        0.0
    };
    
    // 3. Very rare extreme moves (mimicking flash crashes or pumps)
    let black_swan = if rng.r#gen::<f64>() < 0.01 {  // 1% chance of extreme move
        let direction = if rng.r#gen::<bool>() { 1.0 } else { -1.0 };
        direction * rng.r#gen_range(0.08..0.15)  // 8-15% additional move
    } else {
        0.0
    };
    
    // 4. Market trend component (slight bias in one direction that changes occasionally)
    // This is a simplification; real markets have more complex trends
    let trend = (rng.r#gen::<f64>() - 0.48) * 0.01;  // Slight upward bias (0.48 instead of 0.5)
    
    // Combine all components
    let total_change = base_change + rare_event + black_swan + trend;
    
    // Apply the change with a floor to prevent negative prices
    let new_price = current_price * (1.0 + total_change);
    new_price.max(0.01)  // Ensure price never goes below 0.01
}

// Converts USD to crypto (SOL/ETH)
pub fn usd_to_crypto(usd: f64, crypto: &str, price_per_crypto: f64) -> Option<f64> {
    if price_per_crypto <= 0.0 {
        return None; // Invalid price, return None
    }

    let amount_crypto = usd / price_per_crypto;
    Some(amount_crypto)
}

// Converts crypto back to USD
pub fn crypto_to_usd(crypto_amount: f64, price_per_crypto: f64) -> f64 {
    crypto_amount * price_per_crypto
}

// Gets the base price of a cryptocurrency (SOL, ETH)
pub fn get_base_price(crypto: &str) -> f64 {
    match crypto.to_lowercase().as_str() {
        "sol" => 150.0,  // Example base price for SOL
        "eth" => 2000.0, // Example base price for ETH
        _ => 1.0,        // Default to 1.0 if the crypto is not recognized
    }
}