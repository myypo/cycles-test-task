use std::collections::HashSet;

use futures::{stream::FuturesUnordered, StreamExt};
use uuid::Uuid;

use crate::{
    config::EdamamConfig,
    services::fixture::{FixtureBurger, FixtureIngredient, FixtureTag},
};

use super::{Fixture, FixtureClient};

pub struct EdamamClient {
    cli: reqwest::Client,
    conf: EdamamConfig,
}

impl EdamamClient {
    pub fn new(cli: reqwest::Client, conf: EdamamConfig) -> Self {
        Self { cli, conf }
    }
}

#[derive(Clone, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ApiIngredient {
    text: Option<String>,
}
#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ApiRecipe {
    label: Option<String>,
    tags: Option<Vec<String>>,
    image: Option<String>,
    ingredients: Option<Vec<ApiIngredient>>,
}

#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ApiHit {
    recipe: ApiRecipe,
}

#[derive(Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ApiResponse {
    hits: Vec<ApiHit>,
}

impl FixtureClient for EdamamClient {
    async fn fixture(&self) -> anyhow::Result<Fixture> {
        let res = self
            .cli
            .get("https://api.edamam.com/api/recipes/v2")
            .query(&[
                ("type", "any"),
                ("q", "burger"),
                ("app_id", &self.conf.id_app),
                ("app_key", &self.conf.key_app),
                ("random", "true"),
                //
                ("field", "label"),
                ("field", "image"),
                ("field", "tags"),
                ("field", "ingredients"),
            ])
            .send()
            .await?
            .json::<ApiResponse>()
            .await?;

        let tag_list = res
            .hits
            .iter()
            .flat_map(|h| h.recipe.tags.clone().unwrap_or_default())
            .map(|t| t.to_lowercase())
            .collect::<HashSet<String>>()
            .into_iter()
            .map(|name| FixtureTag {
                id: Uuid::new_v4(),
                name,
            })
            .collect::<Vec<FixtureTag>>();
        let ingredient_list = res
            .hits
            .iter()
            .map(|h| h.recipe.ingredients.clone().unwrap_or_default())
            .flat_map(|il| il.into_iter().map(|i| i.text.unwrap_or_default()))
            .map(|i| i.to_lowercase())
            .collect::<HashSet<String>>()
            .into_iter()
            .map(|name| FixtureIngredient {
                id: Uuid::new_v4(),
                name,
            })
            .collect::<Vec<FixtureIngredient>>();

        let burger_list: Vec<FixtureBurger> = {
            res.hits
                .into_iter()
                .map(|h| async {
                    Some(FixtureBurger {
                        name: h.recipe.label.unwrap_or_default(),
                        tag_id_list: tag_list.iter().map(|t| t.id).collect(),
                        ingredient_id_list: ingredient_list.iter().map(|i| i.id).collect(),
                        image_list: vec![{
                            let bytes = self
                                .cli
                                .get(h.recipe.image.unwrap_or_default())
                                .send()
                                .await
                                .ok()?
                                .bytes()
                                .await
                                .ok()?;
                            (Uuid::new_v4(), bytes)
                        }],
                    })
                })
                .collect::<FuturesUnordered<_>>()
                .filter_map(|f| async move { f })
                .collect::<Vec<FixtureBurger>>()
                .await
        };

        Ok(Fixture {
            burger_list,
            tag_list,
            ingredient_list,
        })
    }
}
