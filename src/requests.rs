use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct RegisterForm{
    pub name: String,
    pub password: String,
}
