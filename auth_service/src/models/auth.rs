use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq,JsonSchema, Serialize, Deserialize)]
pub struct Email {
    pub email: String,
    pub expiration_minutes: Option<u32>
}

#[derive(Debug, Clone, PartialEq,JsonSchema, Serialize, Deserialize)]
pub struct Shopify {
    pub email: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq,JsonSchema, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub phone_number: String,
    pub expiration_minutes: Option<u32>
}

impl PhoneNumber {
    pub fn e164_format(&mut self) {
        let mut filter: String = self.phone_number.chars().filter(|c| c.is_digit(10)).collect();                        
        if filter.len() == 0 {
            return;
        }

        let ch = filter.chars().nth(0).unwrap();
        if ch != '+' {
            filter.insert_str(0, "+");
            self.phone_number = filter;
        }
    }
}

#[derive(Debug, Clone, PartialEq,JsonSchema, Serialize, Deserialize)]
pub struct StytchAuth {
    pub token: String,
    pub user_id: Option<String>
}

#[derive(Debug, Clone, PartialEq,JsonSchema, Serialize, Deserialize)]
pub struct StytchToken {
    pub token: String,
}

#[derive(Debug, Clone, PartialEq,JsonSchema, Serialize, Deserialize)]
pub struct StytchOTP {
    pub code: String,
    pub method_id: String,
}