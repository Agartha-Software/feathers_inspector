use bevy::ecs::template::template;
use bevy::prelude::*;
use bevy::reflect::enums::Enum;
use bevy::scene2::{Scene, bsn};
use bevy::ui::Val::*;
use bevy::{
    feathers::{
        theme::{ThemeFontColor, ThemedText},
        tokens,
    },
    platform::collections::HashMap,
};
use core::any::TypeId;

use crate::gui::{config::InspectorConfig, widgets::FieldPath};

use super::reflected::PartialReflectWidget;

pub struct ErasedScene(Box<dyn Scene>);

impl Scene for ErasedScene {
    fn resolve(
        &self,
        context: &mut bevy::scene2::ResolveContext,
        scene: &mut bevy::scene2::ResolvedScene,
    ) -> Result<(), bevy::scene2::ResolveSceneError> {
        self.0.resolve(context, scene)
    }

    fn register_dependencies(&self, dependencies: &mut bevy::scene2::SceneDependencies) {
        self.0.register_dependencies(dependencies);
    }
}

impl ErasedScene {
    pub fn new<S: Scene>(widget: S) -> Self {
        Self(Box::new(widget))
    }
}

/// Type-erasing function that initializes the [`Scene`] for a given widget
type WidgetCreator =
    Box<dyn Fn(&dyn PartialReflect, &FieldPath) -> Option<ErasedScene> + Sync + Send>;

/// Registry storing the widget implementations for types
#[derive(Resource, Default)]
pub struct WidgetRegistry {
    builders: HashMap<TypeId, WidgetCreator>,
}

impl WidgetRegistry {
    pub fn with<T: PartialReflectWidget>(mut self) -> Self {
        self.add::<T>();
        self
    }

    /// Register a type that implements a widget
    pub fn add<T: PartialReflectWidget>(&mut self) {
        self.builders
            .insert(TypeId::of::<T>(), Box::new(Self::builder_for::<T>));
    }

    /// Register or override a type with a widget and bypass the [`PartialReflectWidget`] trait
    pub fn add_custom<T: Reflect>(&mut self, builder: WidgetCreator) {
        self.builders.insert(TypeId::of::<T>(), builder);
    }

    /// get a widget builder for this type if is registered
    pub fn get_widget(
        &self,
        t: &dyn PartialReflect,
        field_path: &FieldPath,
    ) -> Option<ErasedScene> {
        let type_id = t.get_represented_type_info().map(|info| info.type_id());
        type_id
            .and_then(|type_id| self.builders.get(&type_id))
            .and_then(|b| b(t, field_path))
    }

    /// get a label pseudo-widget
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

    /// get an enum variant widget for this type
    /// todo: mutation of the variant with this widget
    pub fn enum_widget(type_name: &str, t: &dyn Enum) -> impl Scene {
        let inner = format!("{type_name}::{}", t.variant_name());

        bsn!(
            Node {
                min_width: Val::Px(60.0),
                padding: UiRect::horizontal(Px(4.0)),
                border: UiRect::all(Px(1.0)),
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

    /// function enclosing the creation of a builder
    fn builder_for<T: PartialReflectWidget>(
        t: &dyn PartialReflect,
        field_path: &FieldPath,
    ) -> Option<ErasedScene> {
        let widget = <T as PartialReflectWidget>::try_widget(t, field_path)?;
        Some(ErasedScene::new(widget))
    }
}
