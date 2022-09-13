use fil_fungible_token::token::TokenError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("error in token: {0}")]
    Token(#[from] TokenError),
}
