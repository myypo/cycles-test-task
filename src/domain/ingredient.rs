#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct Ingredient<I> {
    pub id: I,
    pub name: String,
}
