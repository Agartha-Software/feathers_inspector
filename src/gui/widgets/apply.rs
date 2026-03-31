use bevy::{prelude::*, reflect::ReflectMut};

use crate::{
    gui::widgets::{FieldPath, FieldPathSegment},
    reflection_tools::get_component_reflect_mut,
};

/// Resource to queue value changes for the write-back system
#[derive(Resource, Default)]
pub struct DeferredChanges {
    pub changes: Vec<DeferredChange>,
}

impl DeferredChanges {
    fn system(world: &mut World) {
        world.resource_scope(|world: &mut World, mut self_: Mut<Self>| {
            self_.apply_changes(world);
        })
    }

    fn apply_changes(&mut self, world: &mut World) {
        for change in self.changes.drain(..) {
            let field_path = &change.field_path.clone();
            if let Err(()) = Self::apply_change(world, change) {
                warn!(
                    "Failed to set field value at path {:?} for entity {:?}",
                    field_path.path, field_path.entity
                );
            }
        }
    }

    fn apply_change(world: &mut World, change: DeferredChange) -> Result<(), ()> {
        let field_path = &change.field_path;

        let mut reflected =
            get_component_reflect_mut(world, field_path.entity, field_path.component_type_id)
                .or(Err(()))?;

        Self::apply_recursively(
            reflected.as_partial_reflect_mut(),
            &field_path.path,
            change.patch,
        )
    }

    fn apply_recursively(
        reflected: &mut dyn PartialReflect,
        path: &[FieldPathSegment],
        patch: Box<dyn PartialReflect>,
    ) -> Result<(), ()> {
        let Some(([segment], path)) = path.split_at_checked(1) else {
            return match patch.try_downcast::<f64>() {
                Ok(patch) => Self::apply_numeric(reflected, *patch),
                Err(patch) => reflected.try_apply(patch.as_partial_reflect()).or(Err(())),
            };
        };

        match reflected.reflect_mut() {
            ReflectMut::Struct(s) => {
                if let FieldPathSegment::Named(name) = segment
                    && let Some(field) = s.field_mut(name)
                {
                    return Self::apply_recursively(field, path, patch);
                }
            }
            ReflectMut::TupleStruct(ts) => {
                if let FieldPathSegment::Index(idx) = segment
                    && let Some(field) = ts.field_mut(*idx)
                {
                    return Self::apply_recursively(field, path, patch);
                }
            }
            ReflectMut::Tuple(t) => {
                if let FieldPathSegment::Index(idx) = segment
                    && let Some(field) = t.field_mut(*idx)
                {
                    return Self::apply_recursively(field, path, patch);
                }
            }
            ReflectMut::Enum(e) => {
                if let FieldPathSegment::Index(variant_idx) = segment
                    && variant_idx == &e.variant_index()
                    && let Some(field) = match path.first() {
                        Some(FieldPathSegment::Index(idx)) => e.field_at_mut(*idx),
                        Some(FieldPathSegment::Named(name)) => e.field_mut(name),
                        None => None,
                    }
                {
                    return Self::apply_recursively(field, &path[1..], patch);
                }
            }
            ReflectMut::Array(_) => {}
            ReflectMut::Map(_) => {}
            ReflectMut::List(_) => {}
            ReflectMut::Set(_) => {}
            ReflectMut::Opaque(_) => return Err(()),
        }

        Err(())
    }

    fn apply_numeric(reflected: &mut dyn PartialReflect, new_value: f64) -> Result<(), ()> {
        // Try to apply to f32
        if let Some(f32_val) = reflected.try_downcast_mut::<f32>() {
            *f32_val = new_value as f32;
            return Ok(());
        }

        // Try to apply to f64
        if let Some(f64_val) = reflected.try_downcast_mut::<f64>() {
            *f64_val = new_value;
            return Ok(());
        }

        // Try to apply to i32
        if let Some(i32_val) = reflected.try_downcast_mut::<i32>() {
            *i32_val = new_value as i32;
            return Ok(());
        }

        // Try to apply to i64
        if let Some(i64_val) = reflected.try_downcast_mut::<i64>() {
            *i64_val = new_value as i64;
            return Ok(());
        }

        // Try to apply to u32
        if let Some(u32_val) = reflected.try_downcast_mut::<u32>() {
            *u32_val = new_value.max(0.0) as u32;
            return Ok(());
        }

        // Try to apply to u64
        if let Some(u64_val) = reflected.try_downcast_mut::<u64>() {
            *u64_val = new_value.max(0.0) as u64;
            return Ok(());
        }

        Err(())
    }
}

#[derive(Event)]
pub struct DeferredChange {
    /// The UI entity that triggered this change.
    pub source: Entity,
    /// The field path for write-back.
    pub field_path: FieldPath,
    /// The new value.
    pub patch: Box<dyn PartialReflect>,
}

pub struct DeferredChangePlugin;

impl Plugin for DeferredChangePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeferredChanges>()
            .add_systems(Update, DeferredChanges::system);
    }
}
