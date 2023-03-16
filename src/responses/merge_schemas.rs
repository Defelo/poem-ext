use itertools::Itertools;
use poem_openapi::registry::{MetaMediaType, MetaResponse, MetaSchema, MetaSchemaRef};

pub(super) fn merge_meta_media_types(
    meta_media_types: impl IntoIterator<Item = MetaMediaType>,
) -> Vec<MetaMediaType> {
    meta_media_types
        .into_iter()
        .into_group_map_by(|e| e.content_type)
        .into_iter()
        .map(|(content_type, meta_media_types)| {
            if meta_media_types.len() == 1 {
                meta_media_types.into_iter().next().unwrap()
            } else {
                MetaMediaType {
                    content_type,
                    schema: MetaSchemaRef::Inline(Box::new(MetaSchema {
                        one_of: meta_media_types.into_iter().map(|e| e.schema).collect(),
                        ..MetaSchema::ANY
                    })),
                }
            }
        })
        .collect()
}

pub(super) fn merge_meta_responses(
    responses: impl IntoIterator<Item = MetaResponse>,
) -> Vec<MetaResponse> {
    responses
        .into_iter()
        .into_group_map_by(|e| e.status)
        .into_iter()
        .map(|(status, responses)| {
            if responses.len() == 1 {
                responses.into_iter().next().unwrap()
            } else {
                let description = format!(
                    "There are multiple possible responses with this status code:\n{}",
                    responses
                        .iter()
                        .map(|r| format!("- {}", r.description))
                        .join("\n")
                );
                let mut content = Vec::new();
                let mut headers = Vec::new();
                for response in responses {
                    content.extend(response.content);
                    headers.extend(response.headers);
                }

                MetaResponse {
                    // `Box::leak` is required because `description` has to be a `&'static str`
                    description: Box::leak(description.into_boxed_str()),
                    status,
                    content: merge_meta_media_types(content),
                    headers,
                }
            }
        })
        .collect()
}
