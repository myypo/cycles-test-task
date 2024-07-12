mod create_ingredient;
pub use create_ingredient::create_ingredient;

mod get_ingredient_by_id;
pub use get_ingredient_by_id::get_ingredient_by_id;

mod list_ingredients;
pub use list_ingredients::list_ingredients;

mod add_ingredient_image;
pub use add_ingredient_image::add_ingredient_image;

use sqlx::types::Uuid;
pub type Ingredient = crate::domain::Ingredient<Uuid>;

impl From<(Uuid, String)> for crate::domain::Ingredient<Uuid> {
    fn from(value: (sqlx::types::Uuid, String)) -> Self {
        Self {
            id: value.0,
            name: value.1,
        }
    }
}
