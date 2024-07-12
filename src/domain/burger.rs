use super::{ImageList, Ingredient, Tag};

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct Burger<I> {
    pub id: I,
    pub name: String,
    pub tag_list: Vec<Tag<I>>,
    pub ingredient_list: Vec<Ingredient<I>>,
    pub image_list: ImageList<I>,
}
