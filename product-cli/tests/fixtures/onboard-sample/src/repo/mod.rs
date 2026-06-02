//! Repository layer — sole owner of database access
//! CONSTRAINT: no other module may import sqlx directly
use sqlx;

pub mod users;
pub mod orders;
