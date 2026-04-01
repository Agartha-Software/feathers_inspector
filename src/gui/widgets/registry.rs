use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::scene2::Scene;
use core::any::TypeId;

use crate::gui::widgets::{FieldPath, reflected::PartialReflectWidget};

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
type WidgetCreator = fn(&dyn PartialReflect, &FieldPath) -> Option<ErasedScene>;

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
            .insert(TypeId::of::<T>(), Self::widget_for::<T>);
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

    /// Type erase the widget [`Scene`] for a given type implementing [`PartialReflectWidget`]
    fn widget_for<T: PartialReflectWidget>(
        t: &dyn PartialReflect,
        field_path: &FieldPath,
    ) -> Option<ErasedScene> {
        let widget = <T as PartialReflectWidget>::try_widget(t, field_path)?;
        Some(ErasedScene::new(widget))
    }
}
