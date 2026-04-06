use crate::error::AppError;
use crate::models::{Channel, Program};

pub fn parse_channels(_xml: &str) -> Result<Vec<Channel>, AppError> {
    Ok(vec![])
}

pub fn parse_programs(_xml: &str) -> Result<Vec<Program>, AppError> {
    Ok(vec![])
}
