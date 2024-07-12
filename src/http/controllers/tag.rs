mod create_tag;
pub use create_tag::create_tag;

mod list_tags;
pub use list_tags::list_tags;

use sqlx::types::Uuid;
pub type Tag = crate::domain::Tag<Uuid>;

impl From<(Uuid, String)> for crate::domain::Tag<Uuid> {
    fn from(value: (sqlx::types::Uuid, String)) -> Self {
        Self {
            id: value.0,
            name: value.1,
        }
    }
}
