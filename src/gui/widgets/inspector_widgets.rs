use bevy::ecs::template::template;
use bevy::feathers::theme::{ThemeFontColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::reflect::enums::Enum;
use bevy::scene2::{Scene, bsn};

use crate::gui::config::InspectorConfig;

/// Get a label pseudo-widget
pub fn label_widget(label: String) -> impl Scene {
    bsn!(
        Text::new(label.clone())
        template(move |ctx| {
            let config = ctx.resource::<InspectorConfig>();

            Ok((
                TextFont {
                    font_size: FontSize::Px(config.small_font_size),
                    ..default()
                },
                ThemedText,
                ThemeFontColor(tokens::TEXT_DIM),
                TextColor(config.muted_text_color)
            ))
        })
    )
}

/// Get an enum variant widget for this type
/// todo: mutation of the variant with this widget
pub fn enum_widget(type_name: &str, t: &dyn Enum) -> impl Scene {
    let inner = format!("{type_name}::{}", t.variant_name());

    bsn!(
        Node {
            min_width: Val::Px(60.0),
            padding: UiRect::horizontal(Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
        }
        BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0))
        BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0))
        Children [(
            Text::new(inner.clone())
            template(move |ctx| {
                let small_font_size = ctx.resource::<InspectorConfig>().small_font_size;

                Ok(TextFont {
                    font_size: FontSize::Px(small_font_size),
                    ..default()
                })}
            )
        )]
    )
}
