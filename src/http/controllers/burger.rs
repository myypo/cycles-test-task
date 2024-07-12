mod create_burger;
pub use create_burger::*;

mod get_burger_by_id;
pub use get_burger_by_id::*;

mod list_burgers;
pub use list_burgers::*;

mod random_burgers;
pub use random_burgers::*;

mod add_burger_image;
pub use add_burger_image::*;

pub type Burger = crate::domain::Burger<sqlx::types::Uuid>;
