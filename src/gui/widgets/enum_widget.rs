use bevy::{
    feathers::{font_styles::InheritableFont, theme::ThemedText},
    prelude::*,
    reflect::enums::DynamicEnum,
};

use crate::gui::widgets::{
    apply::{DeferredChange, DeferredChanges},
    dropdown::{DropDown, DropDownEvent, dropdown, dropdown_entries},
};

use super::FieldPath;
use bevy::{ecs::bundle::Bundle, reflect::enums::EnumInfo};

#[derive(Component)]
struct EnumWidget {
    field_path: FieldPath,
    info: EnumInfo,
}

pub fn widget(
    current: usize,
    type_name: &str,
    info: EnumInfo,
    field_path: FieldPath,
    overrides: impl Bundle,
    entry_overrides: impl Bundle + Clone,
) -> impl Bundle {
    (
        Node {
            // min_width: Px(60.0),
            // padding: UiRect::horizontal(Px(4.0)),
            // border: UiRect::all(Px(1.0)),
            ..default()
        },
        // BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0)),
        // BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)),
        Children::spawn((
            Spawn((
                InheritableFont {
                    font_size: FontSize::Px(11.),
                    ..default()
                },
                Text::new(format!("{type_name}::")),
                ThemedText,
                // TextFont {
                //     font_size: FontSize::Px(8.0),
                //     ..default()
                // },
            )),
            Spawn(dropdown(
                Some(current),
                (
                    InheritableFont {
                        font_size: FontSize::Px(11.),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0)),
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0)),
                    EnumWidget {
                        field_path,
                        info: info.clone(),
                    },
                    overrides,
                ),
                dropdown_entries(
                    info.variant_names()
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .collect(),
                    entry_overrides,
                ),
            )),
        )),
    )
}

pub fn make_variant<T: FromReflect>(info: &'static EnumInfo, index: usize) -> Option<T> {
    let variant_name = *info.variant_names().get(index)?;
    let dynamic = match info.variant_at(index).unwrap() {
        bevy::reflect::enums::VariantInfo::Struct(_struct_variant_info) => None,
        bevy::reflect::enums::VariantInfo::Tuple(_tuple_variant_info) => None,
        bevy::reflect::enums::VariantInfo::Unit(_unit_variant_info) => {
            Some(DynamicEnum::new_with_index(index, variant_name, ()))
        }
    }?;

    T::from_reflect(&dynamic)
}

pub fn make_partial_variant(info: &EnumInfo, index: usize) -> Option<Box<dyn PartialReflect>> {
    let variant_name = *info.variant_names().get(index).expect("here");
    match info.variant_at(index).expect("here") {
        bevy::reflect::enums::VariantInfo::Struct(_struct_variant_info) => None,
        bevy::reflect::enums::VariantInfo::Tuple(_tuple_variant_info) => None,
        bevy::reflect::enums::VariantInfo::Unit(_unit_variant_info) => Some(Box::new(
            DynamicEnum::new_with_index(index, variant_name, ()),
        )),
    }
}

fn queue_value_change(
    trigger: On<DropDownEvent>,
    q_dropdown: Query<&EnumWidget, With<DropDown>>,
    mut pending: ResMut<DeferredChanges>,
) {
    println!("got value change");
    if let DropDownEvent::Set(entity, index) = &*trigger {
        println!("got set");
        if let Ok(widget) = q_dropdown.get(*entity) {
            println!("got widget");
            if let Some(patch) = make_partial_variant(&widget.info, *index) {
                println!("got patch");
                pending.changes.push(DeferredChange {
                    source: *entity,
                    field_path: widget.field_path.clone(),
                    patch,
                });
            }
        }
    }
}

pub struct EnumWidgetPlugin;

impl Plugin for EnumWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(queue_value_change);
    }
}
