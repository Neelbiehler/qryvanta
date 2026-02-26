use std::collections::BTreeMap;

use qryvanta_domain::{
    EntityFieldDefinition, FormDefinition, PublishedEntitySchema, ViewDefinition,
};

use crate::dto::{PublishFieldDiffItemResponse, PublishSurfaceDeltaItemResponse};

pub(super) fn compute_field_diff(
    draft_fields: &[EntityFieldDefinition],
    published_schema: Option<&PublishedEntitySchema>,
) -> Vec<PublishFieldDiffItemResponse> {
    let draft_by_name = draft_fields
        .iter()
        .map(|field| (field.logical_name().as_str().to_owned(), field))
        .collect::<BTreeMap<_, _>>();
    let published_by_name = published_schema
        .map(|schema| {
            schema
                .fields()
                .iter()
                .map(|field| (field.logical_name().as_str().to_owned(), field))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut names = draft_by_name.keys().cloned().collect::<Vec<_>>();
    for name in published_by_name.keys() {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    names.sort();

    names
        .into_iter()
        .filter_map(|field_name| {
            let draft = draft_by_name.get(&field_name).copied();
            let published = published_by_name.get(&field_name).copied();

            match (draft, published) {
                (Some(draft_field), None) => Some(PublishFieldDiffItemResponse {
                    field_logical_name: field_name,
                    change_type: "added".to_owned(),
                    draft_field_type: Some(draft_field.field_type().as_str().to_owned()),
                    published_field_type: None,
                    draft_relation_target: draft_field
                        .relation_target_entity()
                        .map(|value| value.as_str().to_owned()),
                    published_relation_target: None,
                }),
                (None, Some(published_field)) => Some(PublishFieldDiffItemResponse {
                    field_logical_name: field_name,
                    change_type: "removed".to_owned(),
                    draft_field_type: None,
                    published_field_type: Some(published_field.field_type().as_str().to_owned()),
                    draft_relation_target: None,
                    published_relation_target: published_field
                        .relation_target_entity()
                        .map(|value| value.as_str().to_owned()),
                }),
                (Some(draft_field), Some(published_field)) => {
                    let type_changed = draft_field.field_type() != published_field.field_type();
                    let relation_changed = draft_field
                        .relation_target_entity()
                        .map(|value| value.as_str())
                        != published_field
                            .relation_target_entity()
                            .map(|value| value.as_str());

                    if !(type_changed || relation_changed) {
                        return None;
                    }

                    Some(PublishFieldDiffItemResponse {
                        field_logical_name: field_name,
                        change_type: "updated".to_owned(),
                        draft_field_type: Some(draft_field.field_type().as_str().to_owned()),
                        published_field_type: Some(
                            published_field.field_type().as_str().to_owned(),
                        ),
                        draft_relation_target: draft_field
                            .relation_target_entity()
                            .map(|value| value.as_str().to_owned()),
                        published_relation_target: published_field
                            .relation_target_entity()
                            .map(|value| value.as_str().to_owned()),
                    })
                }
                (None, None) => None,
            }
        })
        .collect()
}

pub(super) fn compute_form_surface_delta(
    draft_forms: &[FormDefinition],
    published_forms: &[FormDefinition],
) -> Vec<PublishSurfaceDeltaItemResponse> {
    let draft_by_name = draft_forms
        .iter()
        .map(|form| (form.logical_name().as_str().to_owned(), form))
        .collect::<BTreeMap<_, _>>();
    let published_by_name = published_forms
        .iter()
        .map(|form| (form.logical_name().as_str().to_owned(), form))
        .collect::<BTreeMap<_, _>>();

    let mut names = draft_by_name.keys().cloned().collect::<Vec<_>>();
    for name in published_by_name.keys() {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    names.sort();

    names
        .into_iter()
        .map(|logical_name| {
            let draft = draft_by_name.get(&logical_name).copied();
            let published = published_by_name.get(&logical_name).copied();

            let change_type = match (draft, published) {
                (Some(_), None) => "added",
                (None, Some(_)) => "removed",
                (Some(draft_form), Some(published_form)) => {
                    let changed = draft_form.display_name() != published_form.display_name()
                        || count_form_field_placements(draft_form)
                            != count_form_field_placements(published_form)
                        || (draft_form.form_type().as_str() == "main")
                            != (published_form.form_type().as_str() == "main");
                    if changed { "updated" } else { "unchanged" }
                }
                (None, None) => "unchanged",
            }
            .to_owned();

            PublishSurfaceDeltaItemResponse {
                logical_name,
                change_type,
                draft_display_name: draft.map(|value| value.display_name().as_str().to_owned()),
                published_display_name: published
                    .map(|value| value.display_name().as_str().to_owned()),
                draft_item_count: draft.map(count_form_field_placements),
                published_item_count: published.map(count_form_field_placements),
                draft_is_default: draft.map(|value| value.form_type().as_str() == "main"),
                published_is_default: published.map(|value| value.form_type().as_str() == "main"),
            }
        })
        .collect()
}

pub(super) fn compute_view_surface_delta(
    draft_views: &[ViewDefinition],
    published_views: &[ViewDefinition],
) -> Vec<PublishSurfaceDeltaItemResponse> {
    let draft_by_name = draft_views
        .iter()
        .map(|view| (view.logical_name().as_str().to_owned(), view))
        .collect::<BTreeMap<_, _>>();
    let published_by_name = published_views
        .iter()
        .map(|view| (view.logical_name().as_str().to_owned(), view))
        .collect::<BTreeMap<_, _>>();

    let mut names = draft_by_name.keys().cloned().collect::<Vec<_>>();
    for name in published_by_name.keys() {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    names.sort();

    names
        .into_iter()
        .map(|logical_name| {
            let draft = draft_by_name.get(&logical_name).copied();
            let published = published_by_name.get(&logical_name).copied();

            let change_type = match (draft, published) {
                (Some(_), None) => "added",
                (None, Some(_)) => "removed",
                (Some(draft_view), Some(published_view)) => {
                    let changed = draft_view.display_name() != published_view.display_name()
                        || draft_view.columns().len() != published_view.columns().len()
                        || draft_view.is_default() != published_view.is_default();
                    if changed { "updated" } else { "unchanged" }
                }
                (None, None) => "unchanged",
            }
            .to_owned();

            PublishSurfaceDeltaItemResponse {
                logical_name,
                change_type,
                draft_display_name: draft.map(|value| value.display_name().as_str().to_owned()),
                published_display_name: published
                    .map(|value| value.display_name().as_str().to_owned()),
                draft_item_count: draft.map(|value| value.columns().len()),
                published_item_count: published.map(|value| value.columns().len()),
                draft_is_default: draft.map(ViewDefinition::is_default),
                published_is_default: published.map(ViewDefinition::is_default),
            }
        })
        .collect()
}

fn count_form_field_placements(form: &FormDefinition) -> usize {
    form.tabs()
        .iter()
        .map(|tab| {
            tab.sections()
                .iter()
                .map(|section| section.fields().len())
                .sum::<usize>()
        })
        .sum()
}
