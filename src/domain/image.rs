use futures::{stream::FuturesUnordered, StreamExt};
use url::Url;

use crate::services::file_storage::FileClient;

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct Image<I> {
    pub id: I,
    pub url: ImageAvailability,
}

#[derive(Default, serde::Serialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct ImageList<I>(Vec<Image<I>>);

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct WithImageList<T, I> {
    #[serde(flatten)]
    pub inner: T,

    pub image_list: ImageList<I>,
}

impl<I> From<Vec<Image<I>>> for ImageList<I> {
    fn from(val: Vec<Image<I>>) -> Self {
        ImageList(val)
    }
}

impl<I: Clone + Into<String>> ImageList<I> {
    pub async fn fetch_all(fc: &impl FileClient, id_list: impl IntoIterator<Item = I>) -> Self {
        let stream: FuturesUnordered<_> = id_list
            .into_iter()
            .map(|id| async move {
                Image {
                    url: match fc.get_file_url(id.clone()).await {
                        Ok(url) => ImageAvailability::Ok(url),
                        _ => ImageAvailability::Failed,
                    },
                    id,
                }
            })
            .collect();

        Self(stream.collect::<Vec<Image<I>>>().await)
    }
}

#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(untagged, rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImageAvailability {
    #[serde(rename = "OK")]
    Ok(Url),
    Failed,
    NotRequested,
}
