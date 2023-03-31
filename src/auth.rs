use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub const JWT_SECRET: &str = "my-32-character-ultra-secure-12";
pub const JWT_EXPIRES_IN_MINUTES: u128 = 60;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject
    exp: u128,       // expiration
}

impl Claims {
    pub fn new(username: String) -> Self {
        Self {
            sub: username,
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis()
                + JWT_EXPIRES_IN_MINUTES * 60 * 1000,
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        // If subject empty or token expired, return error

        if self.sub.is_empty() {
            return Err("Invalid token".to_string());
        }

        if self.exp
            < SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis()
        {
            return Err("Token expired".to_string());
        }

        Ok(())
    }
}

pub fn create_jwt(username: String) -> String {
    let header = Header::default(); // Default algorithm is HS256
    let claims = Claims::new(username); // Create claims from username
    let key = &EncodingKey::from_secret(JWT_SECRET.as_ref());

    let token = jsonwebtoken::encode(&header, &claims, &key).expect("Failed to create JWT");

    token
}

pub fn decode_jwt(token: &String) -> Result<Claims, String> {
    let key = &DecodingKey::from_secret(JWT_SECRET.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    // If the token or its signature is invalid or the claims fail validation, it will return an error.
    let decoded = jsonwebtoken::decode::<Claims>(&token, &key, &validation);

    match decoded {
        Ok(data) => {
            return Ok(data.claims);
        }
        Err(e) => {
            return Err(e.to_string());
        }
    }
}
