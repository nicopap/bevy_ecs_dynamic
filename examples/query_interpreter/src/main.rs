use bevy::{ecs::component::Components, prelude::*, text::DEFAULT_FONT_HANDLE};
use bevy_simple_text_input::{
    TextInput, TextInputChangeEvent, TextInputPlugin, TextInputSubmitEvent,
};

use eval_parser::expressions;
use interpret::{interpret_submitted_query, Interpreter};
use query_parser::query;
use winnow::Parser;

mod bevy_simple_text_input;
mod eval_parser;
mod interpret;
mod query_parser;
mod spawn_scene;

const EXPR_INPUT: Color = Color::YELLOW;
const QUERY_INPUT: Color = Color::BLUE;
const GOOD_INPUT: Color = Color::GREEN;
const BAD_INPUT: Color = Color::RED;

trait CheckInputField {
    fn validate(value: &str) -> bool;
    const SUBMIT: bool;
}
impl CheckInputField for QueryInputField {
    const SUBMIT: bool = true;
    fn validate(value: &str) -> bool {
        let result = query.parse(value);
        if let Err(err) = &result {
            error!("query input:\n{err}");
        }
        result.is_ok()
    }
}
impl CheckInputField for UpdateInputField {
    const SUBMIT: bool = false;
    fn validate(value: &str) -> bool {
        let result = expressions.parse(value);
        if let Err(err) = &result {
            error!("expression input:\n{err}");
        }
        result.is_ok()
    }
}
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct QueryInputField;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct UpdateInputField;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                level: bevy::log::Level::TRACE,
                filter: "\
          gilrs_core=info,gilrs=info,\
          naga=info,wgpu=error,wgpu_hal=error,\
          bevy_app=info,bevy_render::render_resource::pipeline_cache=info,\
          bevy_render::view::window=info,bevy_ecs::world::entity_ref=info"
                    .to_string(),
            }),
            TextInputPlugin,
            spawn_scene::SpawnScenePlugin,
        ))
        .init_resource::<Interpreter>()
        .register_type::<QueryInputField>()
        .register_type::<UpdateInputField>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                check_valid_input::<QueryInputField>,
                check_valid_input::<UpdateInputField>,
                submit_query,
                interpret_submitted_query,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            let button = |b_color, color, font_color, inactive, no_submit| {
                (
                    NodeBundle {
                        style: Style {
                            width: Val::Px(1200.0),
                            border: UiRect::all(Val::Px(2.0)),
                            padding: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        border_color: BorderColor(b_color),
                        background_color: BackgroundColor(color),
                        ..default()
                    },
                    TextInput {
                        text_style: TextStyle {
                            font_size: 13.,
                            font: DEFAULT_FONT_HANDLE.typed(),
                            color: font_color,
                        },
                        inactive,
                        no_submit,
                        ..default()
                    },
                )
            };
            let (white, black) = (Color::WHITE, Color::BLACK);
            parent.spawn((
                UpdateInputField,
                button(GOOD_INPUT, EXPR_INPUT, black, true, true),
            ));
            parent.spawn((
                QueryInputField,
                button(BAD_INPUT, QUERY_INPUT, white, false, false),
            ));
        });
}

fn check_valid_input<I: Component + CheckInputField>(
    mut events: EventReader<TextInputChangeEvent>,
    mut query_input: Query<(&mut TextInput, &mut BorderColor), With<I>>,
) {
    for TextInputChangeEvent { entity, value } in events.iter() {
        let Ok((mut input, mut b_color)) = query_input.get_mut(*entity) else {
            continue;
        };
        if I::validate(value) {
            if input.no_submit && I::SUBMIT {
                input.no_submit = false;
            }
            b_color.0 = GOOD_INPUT;
        } else {
            if !input.no_submit && I::SUBMIT {
                input.no_submit = true;
            }
            b_color.0 = BAD_INPUT;
        }
    }
}
fn submit_query(
    mut events: EventReader<TextInputSubmitEvent>,
    mut interpreter: ResMut<Interpreter>,
    query_input: Query<(), With<QueryInputField>>,
    update_input: Query<Entity, With<UpdateInputField>>,
    children: Query<&Children>,
    text: Query<&Text>,
    reg: Res<AppTypeRegistry>,
    comps: &Components,
) {
    for event in events.iter() {
        if !query_input.contains(event.entity) {
            continue;
        }
        let update = match update_input.get_single() {
            Ok(entity) => {
                let child = children.get(entity).unwrap()[0];
                let child = children.get(child).unwrap()[0];
                text.get(child).map_or(default(), |t| {
                    format!("{}{}", &t.sections[0].value, &t.sections[2].value)
                })
            }
            Err(err) => {
                debug!("no children: {err}");
                continue;
            }
        };
        let Ok(update) = expressions.parse(update.as_str()) else {
            debug!("Not correct expression: {update}");
            continue;
        };
        let query_str = query.parse(&event.value).unwrap();
        info!("{query_str:?}");
        interpreter.set_update(dbg!(update)).unwrap();
        interpreter.set_query(query_str, &reg.read(), comps);
    }
}
