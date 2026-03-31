use bevy::prelude::*;
use bevy::ecs::relationship::{RelatedSpawner, Relationship};
use bevy::ecs::spawn::SpawnableList;
use bevy::input_focus::InputFocus;

use bevy::{feathers::theme::ThemedText, window::SystemCursorIcon};

use bevy::feathers::{
    constants::size,
    cursor::EntityCursor,
};

use crate::gui::widgets::PauseForEditing;

///
/// Hierarchy:
/// Dropdown {
///     DropHeader {
///         DropEntry
///     }
///     DropList {
///         DropEntry
///         DropEntry
///     }
/// }

#[derive(Event, Clone, Debug)]
pub enum DropDownEvent {
    Opened(Entity),
    Set(Entity, usize),
    Reset(Entity),
}

#[derive(Component, Default, Debug)]
pub struct DropDown;

#[derive(Component, Default, Debug)]
pub struct DropList;

#[derive(Component, Debug)]
pub struct DropEntry {
    pub entity: Entity,
    pub index: usize,
}

#[derive(Component, Debug)]
pub struct DropHeader;

/// todo: use Scene trait for chilren ?
pub fn dropdown<B: Bundle>(
    current: Option<usize>,
    overrides: B,
    children: impl SpawnableList<ChildOf> + Send + Sync + 'static,
) -> impl Bundle {
    (
        Node {
            height: size::ROW_HEIGHT,
            flex_direction: FlexDirection::Column,
            // border_radius: props.corners.to_border_radius(4.0),
            ..Default::default()
        },
        Visibility::Inherited,
        // Interaction::default(),
        // Hovered::default(),
        // EntityCursor::System(SystemCursorIcon::Pointer),
        // TabIndex(0),
        // Text(variants.get(current).cloned().unwrap_or(String::new())),
        DropDown,
        PauseForEditing(false),
        // ThemedText,
        // ThemeBackgroundColor(tokens::BUTTON_BG),
        // ThemeFontColor(tokens::BUTTON_TEXT),
        // InheritableFont {
        //     font: HandleOrPath::Path(fonts::REGULAR.to_owned()),
        //     font_size: FontSize::Px(14.0),
        //     weight: FontWeight::NORMAL,
        // },
        overrides,
        Children::spawn((SpawnWith(move |spawner: &mut RelatedSpawner<ChildOf>| {
            let dropdown = spawner.target_entity();

            let header = spawner
                .spawn((
                    Node {
                        min_height: size::ROW_HEIGHT,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        ..Default::default()
                    },
                    Visibility::Inherited,
                    DropHeader,
                ))
                .id();

            let droplist = spawner
                .spawn((
                    Node {
                        top: Val::Percent(100.),
                        width: Val::Percent(100.),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    Visibility::Hidden,
                    DropList,
                    GlobalZIndex(100),
                    Children::spawn(children),
                ))
                .id();

            let world = spawner.world_mut();

            if let Ok(children) = world
                .query::<&Children>()
                .get(&world, droplist)
                .map(|c| c.to_vec())
            {
                children.iter().enumerate().for_each(|(index, child)| {
                    let value = if Some(index) == current {
                        Some(world.entity_mut(*child).clone_and_spawn())
                    } else {
                        None
                    };
                    world.entity_mut(*child).insert(DropEntry {
                        entity: dropdown,
                        index,
                    });

                    if let Some(value) = value {
                        world.entity_mut(header).add_child(value);
                    }
                });
            }
            // parent.world_mut()
        }),)),
    )
}

pub fn dropdown_entries<R: Relationship>(
    labels: Vec<String>,
    overrides: impl Bundle + Clone,
) -> impl SpawnableList<R> {
    SpawnIter(labels.into_iter().map(move |label| {
        (
            Node {
                min_height: size::ROW_HEIGHT,
                ..Default::default()
            },
            // Button,
            // Hovered::default(),
            Interaction::default(),
            EntityCursor::System(SystemCursorIcon::Pointer),
            // TabIndex(0),
            // DropEntry { entity, index },
            Text(label.into()),
            ThemedText,
            // ThemeBackgroundColor(tokens::BUTTON_BG),
            // ThemeFontColor(tokens::BUTTON_TEXT),
            // InheritableFont {
            //     font: HandleOrPath::Path(fonts::REGULAR.to_owned()),
            //     font_size: FontSize::Px(14.0),
            //     weight: FontWeight::NORMAL,
            // },
            overrides.clone(),
        )
    }))
}

fn dropdown_set_value(
    trigger: On<DropDownEvent>,
    // dropdown: Entity,
    // index: usize,
    q_header: Query<Entity, With<DropHeader>>,
    q_entry: Query<(Entity, &DropEntry)>,
    q_dropdown: Query<(&DropDown, &Children)>,
    mut commands: Commands,
) {
    if let DropDownEvent::Set(dropdown, index) = *trigger {
        let Ok((_drop, children)) = q_dropdown.get(dropdown) else {
            return;
        };

        let Some(header) = q_header.iter_many(&*children).next() else {
            return;
        };
        commands.entity(header).despawn_children();

        let Some(entry) = q_entry.into_iter().find_map(|(entity, entry)| {
            if entry.entity == dropdown && entry.index == index {
                Some(entity)
            } else {
                None
            }
        }) else {
            return;
        };

        let entry = commands
            .entity(entry)
            .clone_and_spawn()
            .remove::<DropEntry>()
            .id();
        commands.entity(header).add_child(entry);
    }
}

// use bevy::ui_widgets::MenuButton

// pub fn droplist(entity: Entity, source: &DropDown, overrides: impl Bundle + Clone) -> impl Bundle {
//     (
//         Node {
//             top: size::ROW_HEIGHT,
//             height: size::ROW_HEIGHT,
//             justify_content: JustifyContent::FlexStart,
//             align_items: AlignItems::FlexStart,
//             padding: UiRect::axes(Val::Px(8.0), Val::Px(0.)),
//             display: Display::Flex,
//             flex_direction: FlexDirection::Column,
//             flex_grow: 1.0,
//             position_type: PositionType::Absolute,
//             // border_radius: props.corners.to_border_radius(4.0),
//             ..Default::default()
//         },
//         Visibility::Visible,
//         DropList,
//         GlobalZIndex(100),
//         Children::spawn(SpawnIter(source.0.clone().into_iter().enumerate().map(
//             move |(index, label)| {
//                 (
//                     Node {
//                         height: size::ROW_HEIGHT,
//                         padding: UiRect::axes(Val::Px(8.0), Val::Px(0.)),
//                         flex_grow: 1.0,
//                         // border_radius: props.corners.to_border_radius(4.0),
//                         ..Default::default()
//                     },
//                     // Button,
//                     // Hovered::default(),
//                     Interaction::default(),
//                     EntityCursor::System(SystemCursorIcon::Pointer),
//                     // TabIndex(0),
//                     DropEntry { entity, index },
//                     Text(label),
//                     ThemedText,
//                     ThemeBackgroundColor(tokens::BUTTON_BG),
//                     ThemeFontColor(tokens::BUTTON_TEXT),
//                     InheritableFont {
//                         font: HandleOrPath::Path(fonts::REGULAR.to_owned()),
//                         font_size: FontSize::Px(14.0),
//                         weight: FontWeight::NORMAL,
//                     },
//                     overrides.clone(),
//                 )
//             },
//         ))),
//     )
// }

// Observer: handle click on dropdown or entries in the list
fn dropdown_on_click(
    mut click: On<Pointer<Click>>,
    mut q_dropdown: Query<(&DropDown, &Children)>,
    q_list: Query<(&Visibility, &ChildOf), With<DropList>>,
    q_entry: Query<&DropEntry>,
    mut input_focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    if let Ok((_dropdown, children)) = q_dropdown.get_mut(click.entity) {
        click.propagate(false);

        let is_open = children.iter().any(|child| {
            q_list
                .get(child)
                .map(|(vis, _)| vis != Visibility::Hidden)
                .unwrap_or_default()
        });
        if is_open {
            input_focus.set(click.entity);
            commands.trigger(DropDownEvent::Reset(click.entity));
        } else {
            commands.trigger(DropDownEvent::Opened(click.entity));
        }
    } else if let Ok(entry) = q_entry.get(click.entity) {
        click.propagate(false);
        input_focus.set(entry.entity);
        commands.trigger(DropDownEvent::Set(entry.entity, entry.index));
    } else {
        for (vis, childof) in q_list {
            if vis != Visibility::Hidden {
                commands.trigger(DropDownEvent::Reset(childof.parent()))
            }
        }
    }
}

fn on_trigger(
    trigger: On<DropDownEvent>,
    mut q_droplist: Query<&mut Visibility, With<DropList>>,
    mut q_dropdown: Query<(&DropDown, &mut PauseForEditing, &Children)>,
    mut input_focus: ResMut<InputFocus>,
) {
    match *trigger {
        DropDownEvent::Opened(entity) => {
            if let Ok((_dropdown, mut pause, children)) = q_dropdown.get_mut(entity) {
                children.iter().find_map(|child| {
                    q_droplist.get_mut(child).ok().and_then(|mut visibility| {
                        *visibility = Visibility::Inherited;
                        **pause = true;
                        input_focus.set(child);
                        Some(())
                    })
                });
            }
        }
        DropDownEvent::Set(entity, _index) => {
            if let Ok((_dropdown, mut pause, children)) = q_dropdown.get_mut(entity) {
                children.iter().find_map(|child| {
                    q_droplist.get_mut(child).ok().and_then(|mut visibility| {
                        *visibility = Visibility::Hidden;
                        **pause = false;
                        input_focus.set(entity);
                        Some(())
                    })
                });
                // commands.run_system_cached_with();
                // **pause = false;
                // children
                //     .iter()
                //     .find(|child| q_droplist.get(*child).is_ok())
                //     .map(|child| {
                //         commands.entity(child).despawn();
                //     });
                // text.0 = dropdown.0.get(index).cloned().unwrap_or_default();
            }
        }
        DropDownEvent::Reset(entity) => {
            if let Ok((_dropdown, mut pause, children)) = q_dropdown.get_mut(entity) {
                children.iter().find_map(|child| {
                    q_droplist.get_mut(child).ok().and_then(|mut visibility| {
                        *visibility = Visibility::Hidden;
                        **pause = false;
                        input_focus.set(entity);
                        Some(())
                    })
                });
                // children
                //     .iter()
                //     .find(|child| q_droplist.get(*child).is_ok())
                //     .map(|child| {
                //         commands.entity(child).despawn();
                //     });
            }
        }
    }
}

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(dropdown_on_click)
            .add_observer(on_trigger)
            .add_observer(dropdown_set_value);
    }
}

// pub fn poop(variants: Vec<String>, current: usize) -> impl Bundle {
//     (
//         Node {
//             ..Default::default()
//         },
//         observe(on_menu_event),
//         children![(
//             Node {
//                 width: px(200),
//                 height: px(65),
//                 border: UiRect::all(px(5)),
//                 box_sizing: BoxSizing::BorderBox,
//                 justify_content: JustifyContent::SpaceBetween,
//                 align_items: AlignItems::Center,
//                 padding: UiRect::axes(px(16), px(0)),
//                 border_radius: BorderRadius::all(px(5)),
//                 ..default()
//             },
//             ThemedText,
//             ThemeBackgroundColor(tokens::BUTTON_BG),
//             ThemeFontColor(tokens::BUTTON_TEXT),
//             InheritableFont {
//                 font: HandleOrPath::Path(fonts::REGULAR.to_owned()),
//                 font_size: FontSize::Px(14.0),
//                 weight: FontWeight::NORMAL,
//             },
//             children![(
//                 Node {
//                     height: size::ROW_HEIGHT,
//                     display: Display::Flex,
//                     padding: UiRect::axes(Val::Px(8.0), Val::Px(0.)),
//                     border: UiRect::all(px(1)),
//                     position_type: PositionType::Absolute,
//                     // flex_grow: 1.0,
//                     // border_radius: props.corners.to_border_radius(4.0),
//                     ..Default::default()
//                 },
//                 Interaction::default(),
//                 // Hovered::default(),
//                 EntityCursor::System(SystemCursorIcon::Pointer),
//                 // TabIndex(0),
//                 // Text(variants.get(current).cloned().unwrap_or(String::new())),
//                 PauseForEditing(false),
//                 MenuPopup::default(),
//                 Visibility::Hidden,
//                 Popover {
//                     positions: vec![
//                         PopoverPlacement {
//                             side: PopoverSide::Bottom,
//                             align: PopoverAlign::Start,
//                             gap: 2.0,
//                         },
//                         PopoverPlacement {
//                             side: PopoverSide::Top,
//                             align: PopoverAlign::Start,
//                             gap: 2.0,
//                         },
//                     ],
//                     window_margin: 10.0,
//                 },
//                 // overrides,
//                 Children::spawn(SpawnIter(variants.into_iter().map(|variant| {
//                     (
//                         Node {
//                             height: size::ROW_HEIGHT,
//                             justify_content: JustifyContent::Center,
//                             align_items: AlignItems::Start,
//                             ..Default::default()
//                         },
//                         // Button,
//                         // Hovered::default(),
//                         Interaction::default(),
//                         EntityCursor::System(SystemCursorIcon::Pointer),
//                         // TabIndex(0),
//                         MenuItem,
//                         children![(
//                             // DropEntry { entity, index },
//                             Text(variant),
//                             ThemedText,
//                             ThemeBackgroundColor(tokens::BUTTON_BG),
//                             ThemeFontColor(tokens::BUTTON_TEXT),
//                             InheritableFont {
//                                 font: HandleOrPath::Path(fonts::REGULAR.to_owned()),
//                                 font_size: FontSize::Px(14.0),
//                                 weight: FontWeight::NORMAL,
//                             }
//                         )],
//                     )
//                 }))),
//             )]
//         )],
//     )
// }
