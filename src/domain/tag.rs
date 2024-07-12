#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct Tag<I> {
    pub id: I,
    pub name: String,
}
