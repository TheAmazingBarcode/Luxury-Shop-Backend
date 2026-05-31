use jsonwebtoken::errors::Error;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::OnceLock;

pub static UTIL: OnceLock<JwtUtil> = OnceLock::new();
pub struct JwtUtil {
    decode_key: DecodingKey,
    encode_key: EncodingKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub token_type: String,
}

impl JwtUtil {
    fn new() -> Self {
        let encode_key =
            EncodingKey::from_rsa_pem(fs::read("src/keys/private_key.pem").unwrap().as_slice())
                .expect("PRIVATE KEY INCORRECTLY LOADED");
        let decode_key =
            DecodingKey::from_rsa_pem(fs::read("src/keys/public_key.pem").unwrap().as_slice())
                .expect("PUBLIC KEY INCORRECTLY LOADED");

        Self {
            decode_key,
            encode_key,
        }
    }

    pub fn decode(&self, token: String) -> Result<TokenData<Claims>, Error> {
        jsonwebtoken::decode::<Claims>(&token, &self.decode_key, &Validation::new(Algorithm::RS256))
    }

    pub fn encode(&self, claims: &Claims) -> String {
        jsonwebtoken::encode(&Header::new(Algorithm::RS256), claims, &self.encode_key)
            .expect("JWT INCORRECTLY ENCODED")
    }
}

pub async fn load_jwt() {
    UTIL.get_or_init(JwtUtil::new);
}
