use crate::gui::config::InspectorConfig;
use bevy::ecs::template::template;
use bevy::prelude::*;
use bevy::scene2::{Scene, bsn};
use bevy::ui::Val::*;

use super::drag_value::{DragValue, DragValueDragState};
use super::{FieldPath, FieldPathSegment};

/// Trait to construct a reflector widget from a concrete type
/// This is a convenience trait to enable using a &Self input
///
/// This trait is used to automatically drive the [`PartialReflectWidget`] used in the broader case
/// that the value may be a partially reflected dynamic type
pub trait ReflectWidget: Reflect + Sized {
    /// Construct a widget bundle with a reference to [`Self`] and the path to the field containing self
    /// This bundle will be added to an otherwise empty entity serving as the container for this widget
    fn widget(&self, field_path: &FieldPath) -> impl Scene;
}

/// Trait to construct a reflector widget from any PartialReflect type
/// Implement this trait directly if you need to use it
/// for example if your type doesn't support cloning or full reflection
///
/// usually you can rely on the simpler [`ReflectWidget`] implementing it for you
pub trait PartialReflectWidget: PartialReflect + Sized {
    /// Try to construct a bundle with a potentially dynamic object.
    /// this can fail, for example if this expects the type to be fully reflected but it as not
    fn try_widget(self_: &dyn PartialReflect, field_path: &FieldPath) -> Option<impl Scene>;
}

impl<T: ReflectWidget> PartialReflectWidget for T {
    /// Try using the concrete type's [`ReflectWidget`] implementation
    fn try_widget(self_: &dyn PartialReflect, field_path: &FieldPath) -> Option<impl Scene> {
        self_
            .try_downcast_ref::<Self>()
            .map(|self_| ReflectWidget::widget(self_, field_path))
    }
}

fn vec3_value(field: &str, val: f32, color: Color, mut field_path: FieldPath) -> impl Scene {
    field_path
        .path
        .push(FieldPathSegment::Named(field.to_owned()));

    bsn! {
        Node {
            min_width: Val::Px(60.0),
            padding: UiRect::horizontal(Px(4.0)),
            border: UiRect::all(Px(1.0)),
        }
        BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0))
        BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0))
        template(move |_ctx| Ok((DragValue {
                field_path: field_path.clone(),
                drag_speed: 0.1,
                precision: 2,
                min: None,
                max: None,
            },
        )))
        DragValueDragState
        Interaction
        Children [
            Text::new(format!("{:.2}", val))
            template(move |ctx| {
                let small_font_size = ctx.resource::<InspectorConfig>().small_font_size;

                Ok(TextFont {
                    font_size: FontSize::Px(small_font_size),
                    ..default()
                })
            })
            TextColor(color)
        ]
    }
}

impl PartialReflectWidget for Vec3 {
    fn try_widget(self_: &dyn PartialReflect, field_path: &FieldPath) -> Option<impl Scene> {
        let self_ = self_.reflect_ref().as_struct().ok()?;
        let x = *self_.field_at(0)?.try_downcast_ref::<f32>()?;
        let y = *self_.field_at(1)?.try_downcast_ref::<f32>()?;
        let z = *self_.field_at(2)?.try_downcast_ref::<f32>()?;

        Some(bsn! {
            Node {
                min_width: Val::Px(60.0),
                padding: UiRect::horizontal(Px(4.0)),
                border: UiRect::all(Px(1.0))
            }

            Children [
                (vec3_value("x", x, Color::srgba(1., 0.5, 0.5, 1.), field_path.clone())),
                (vec3_value("y", y, Color::srgba(0.5, 1., 0.5, 1.), field_path.clone())),
                (vec3_value("z", z, Color::srgba(0.5, 0.5, 1., 1.), field_path.clone()))
            ]
        })
    }
}

impl ReflectWidget for f32 {
    fn widget(&self, field_path: &FieldPath) -> impl Scene {
        let val = *self;
        let field_path = field_path.clone();

        bsn!(
            Node {
                min_width: Val::Px(60.0),
                padding: UiRect::horizontal(Px(4.0)),
                border: UiRect::all(Px(1.0)),
            }
            BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 1.0))
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 1.0))
            template(move |_ctx| Ok((
                DragValue {
                    field_path: field_path.clone(),
                    drag_speed: 0.1,
                    precision: 2,
                    min: None,
                    max: None,
                },
            )))
            DragValueDragState
            Interaction
            Children[(
                Text::new(format!("{:.2}", val))
                template(move |ctx| {
                    let small_font_size = ctx.resource::<InspectorConfig>().small_font_size;

                    Ok(TextFont {
                        font_size: FontSize::Px(small_font_size),
                        ..default()
                    })
                })
                TextColor(Color::srgba(0.9, 0.9, 0.6, 1.0))
            )]
        )
    }
}
