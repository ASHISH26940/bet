use actix::{Actor, StreamHandler, AsyncContext};
use actix_web_actors::ws;
use serde::{Serialize, Deserialize};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use crate::game::{Game, Bet};
use crate::crypto::{usd_to_crypto, simulate_price, get_base_price, crypto_to_usd};

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    #[serde(rename = "start")]
    Start { amount: String, crypto: String },
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "set_price")]
    SetPrice { crypto: String, price: f64 },
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerMsg {
    #[serde(rename = "price_update")]
    PriceUpdate { price: f64, multiplier: f64, usd_value: f64 },
    #[serde(rename = "cashout_result")]
    CashoutResult { 
        balance: String,
        crypto_amount: f64,
        usd_amount: f64,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

pub struct WsSession {
    user_id: String,
    game: Arc<Mutex<Game>>,
    crypto: Option<String>,
    price: f64,
    start_time: Option<Instant>,
    bet_amount: Option<f64>, // Track the bet amount in crypto
    initial_usd_value: Option<f64>, // Track the initial USD value
}

impl WsSession {
    pub fn new(game: Arc<Mutex<Game>>) -> Self {
        let user_id = {
            let mut game_lock = game.lock().unwrap();
            game_lock.register_user()
        };

        WsSession {
            user_id,
            game,
            crypto: None,
            price: 0.0,
            start_time: None,
            bet_amount: None,
            initial_usd_value: None,
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Send price updates every second
        ctx.run_interval(Duration::from_secs(1), |act, ctx| {
            if let Some(ref crypto) = act.crypto {
                // Simulate price fluctuation (up or down)
                act.price = simulate_price(act.price);
                
                let multiplier = act.start_time.map(|t| {
                    1.0 + (t.elapsed().as_secs_f64() * 0.01)
                }).unwrap_or(1.0);

                // Calculate current USD value based on original bet
                let current_usd_value = act.bet_amount
                    .map(|amount| amount * multiplier * act.price)
                    .unwrap_or(0.0);
                
                // Ensure the USD value is at least the initial value
                let usd_value = match act.initial_usd_value {
                    Some(initial) => current_usd_value.max(initial),
                    None => current_usd_value,
                };
                
                let msg = ServerMsg::PriceUpdate {
                    price: act.price,
                    multiplier,
                    usd_value,
                };
                ctx.text(serde_json::to_string(&msg).unwrap());
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let Ok(ws::Message::Text(text)) = msg else { return };

        let parsed: Result<ClientMsg, _> = serde_json::from_str(&text);
        match parsed {
            Ok(ClientMsg::Start { amount, crypto }) => {
                let Ok(usd) = amount.parse::<f64>() else {
                    ctx.text(serde_json::to_string(&ServerMsg::Error {
                        message: "Invalid amount".to_string(),
                    }).unwrap());
                    return;
                };

                let price = get_base_price(&crypto);
                let Some(amt_crypto) = usd_to_crypto(usd, &crypto, price) else {
                    ctx.text(serde_json::to_string(&ServerMsg::Error {
                        message: "Conversion failed".to_string(),
                    }).unwrap());
                    return;
                };

                let bet = Bet {
                    crypto: crypto.clone(),
                    amount_crypto: amt_crypto,
                    start_time: Instant::now(),
                };

                let placed = self.game.lock().unwrap().place_bet(&self.user_id, bet);
                if !placed {
                    ctx.text(serde_json::to_string(&ServerMsg::Error {
                        message: "Insufficient balance".to_string(),
                    }).unwrap());
                    return;
                }

                self.crypto = Some(crypto);
                self.price = price;
                self.start_time = Some(Instant::now());
                self.bet_amount = Some(amt_crypto);
                self.initial_usd_value = Some(usd);
            }

            Ok(ClientMsg::Stop) => {
                let multiplier = self.start_time.map(|t| {
                    1.0 + (t.elapsed().as_secs_f64() * 0.01)
                }).unwrap_or(1.0);

                let price_per_crypto = match self.crypto.as_deref() {
                    Some("sol") => self.price, // Use current price from simulation
                    Some("eth") => self.price, // Use current price from simulation
                    _ => {
                        ctx.text(serde_json::to_string(&ServerMsg::Error {
                            message: "Unsupported crypto type".to_string(),
                        }).unwrap());
                        return;
                    }
                };

                // Calculate crypto amount won
                let crypto_amount = match (self.bet_amount, self.crypto.as_deref()) {
                    (Some(amount), Some(_)) => amount * multiplier,
                    _ => {
                        ctx.text(serde_json::to_string(&ServerMsg::Error {
                            message: "No active bet".to_string(),
                        }).unwrap());
                        return;
                    }
                };
                
                // Convert crypto to USD at current price
                let calculated_usd = crypto_to_usd(crypto_amount, price_per_crypto);
                
                // Ensure the USD amount is at least the initial value and never negative
                let usd_amount = match self.initial_usd_value {
                    Some(initial) => calculated_usd.max(initial).max(0.0),
                    None => calculated_usd.max(0.0),
                };

                // Cash out and get new balance
                let Some(new_bal) = self.game.lock().unwrap().cash_out(
                    &self.user_id, 
                    multiplier, 
                    price_per_crypto
                ) else {
                    ctx.text(serde_json::to_string(&ServerMsg::Error {
                        message: "No active bet".to_string(),
                    }).unwrap());
                    return;
                };

                self.crypto = None;
                self.start_time = None;
                self.bet_amount = None;
                self.initial_usd_value = None;

                ctx.text(serde_json::to_string(&ServerMsg::CashoutResult {
                    balance: format!("{:.2}", new_bal),
                    crypto_amount,
                    usd_amount,
                }).unwrap());
            }

            // Handle setting crypto price via payload
            Ok(ClientMsg::SetPrice { crypto, price }) => {
                let mut game_lock = self.game.lock().unwrap();
                game_lock.set_crypto_price(&crypto, price);
                
                // Update local price if this is the active crypto
                if self.crypto.as_deref() == Some(&crypto) {
                    self.price = price;
                }
                
                let usd_value = self.bet_amount
                    .map(|amount| amount * price)
                    .unwrap_or(0.0);
                
                ctx.text(serde_json::to_string(&ServerMsg::PriceUpdate {
                    price,
                    multiplier: 1.0, 
                    usd_value,
                }).unwrap());
            }

            Err(_) => {
                ctx.text(serde_json::to_string(&ServerMsg::Error {
                    message: "Invalid message".to_string(),
                }).unwrap());
            }
        }
    }
}