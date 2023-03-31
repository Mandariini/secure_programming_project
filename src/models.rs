use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub struct UserInfo {
    pub username: String,
    pub hashed_password: Vec<u8>, // Vec<u8> is a dynamic byte array
    pub salt: [u8; 32],           // 32 bytes salt
}

impl UserInfo {
    pub fn create_user(username: String, password: String) -> Self {
        // OsRNG is a random number generator that retrieves randomness from the operating system.
        let mut salt = [0u8; 32]; // 32 bytes
        OsRng.fill_bytes(&mut salt); // Generates a sequence of random bytes

        let hashed_password = Self::hash_password(&password, &salt);

        Self {
            username: username,
            hashed_password: hashed_password,
            salt: salt,
        }
    }

    fn hash_password(password: &String, salt: &[u8]) -> Vec<u8> {
        // Concatenate the password and salt
        let mut input = Vec::with_capacity(password.len() + salt.len());
        input.extend_from_slice(password.as_bytes());
        input.extend_from_slice(salt);

        // Hash the concatenated input using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&input);
        hasher.finalize().to_vec()
    }

    pub fn verify_password(&self, password_to_check: &str) -> bool {
        // Concatenate the password and salt
        let mut input = Vec::with_capacity(password_to_check.len() + self.salt.as_slice().len());
        input.extend_from_slice(password_to_check.as_bytes());
        input.extend_from_slice(self.salt.as_slice());

        // Hash the concatenated input using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(&input);
        let computed_hash = hasher.finalize();

        // Compare the computed hash to the stored hash
        computed_hash.as_slice() == self.hashed_password.as_slice()
    }
}

#[derive(Serialize, Deserialize)]
pub struct RegisterLoginRequest {
    pub username: String,
    pub password: String,
}

impl RegisterLoginRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.username.len() < 4 {
            return Err("Username must be at least 4 characters long.".to_string());
        }

        if self.password.len() < 8 {
            return Err("Password must be at least 8 characters long.".to_string());
        }

        Ok(())
    }
}

#[derive(Serialize)]
pub struct RegisterLoginResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
}
