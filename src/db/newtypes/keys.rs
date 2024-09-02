use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};

/// User ID
#[derive(DieselNewType)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i32);
