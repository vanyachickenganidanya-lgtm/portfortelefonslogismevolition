//! Bevy рисует схему. Логика элементов — Funo (`funo/components`).

use bevy::prelude::*;
use logismevo_sim as sim;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Resource)]
struct WorldSim {
    circuit: sim::Circuit,
    lib: sim::Library,
    tool: String,
    running: bool,
    history: Vec<String>,
    next_id: u32,
}

fn main() {
    let lib = sim::Library::load_dir(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../funo/components"),
    )
    .unwrap_or(sim::Library {
        comps: Default::default(),
    });

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "LogismEvo Phone".into(),
                resolution: (390., 844.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(WorldSim {
            circuit: demo_circuit(),
            lib,
            tool: "and".into(),
            running: true,
            history: vec![],
            next_id: 10,
        })
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(280)))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, tick_sim)
        .add_systems(Update, (draw_circuit, handle_touch))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn tick_sim(mut world: ResMut<WorldSim>) {
    if !world.running {
        return;
    }
    world.circuit.tick(&world.lib);
    let snap: String = world
        .circuit
        .nodes
        .iter()
        .map(|n| format!("{}{}", n.kind.chars().next().unwrap_or('?'), n.value))
        .collect::<Vec<_>>()
        .join(" ");
    world.history.push(snap);
    if world.history.len() > 12 {
        world.history.remove(0);
    }
}

fn draw_circuit(world: Res<WorldSim>, mut gizmos: Gizmos) {
    for w in &world.circuit.wires {
        let a = world.circuit.nodes.iter().find(|n| n.id == w.from);
        let b = world.circuit.nodes.iter().find(|n| n.id == w.to);
        if let (Some(a), Some(b)) = (a, b) {
            let col = if a.value == 1 {
                Color::srgb(0.2, 1.0, 0.45)
            } else {
                Color::srgb(0.35, 0.4, 0.55)
            };
            gizmos.line_2d(Vec2::new(a.x, a.y), Vec2::new(b.x, b.y), col);
        }
    }
    for n in &world.circuit.nodes {
        let col = if n.value == 1 {
            Color::srgb(0.15, 0.95, 0.55)
        } else {
            Color::srgb(0.25, 0.35, 0.7)
        };
        gizmos.circle_2d(Vec2::new(n.x, n.y), 22.0, col);
    }
}

fn handle_touch(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<WorldSim>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, gt)) = camera_q.get_single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(gt, pos) else {
        return;
    };

    if let Some(n) = world.circuit.nodes.iter_mut().find(|n| {
        n.kind == "pin" && (n.x - world_pos.x).hypot(n.y - world_pos.y) < 28.0
    }) {
        n.value ^= 1;
        return;
    }

    let id = world.next_id;
    world.next_id += 1;
    world.circuit.nodes.push(sim::Node {
        id,
        kind: world.tool.clone(),
        x: world_pos.x,
        y: world_pos.y,
        value: 0,
        extra: Default::default(),
    });
}

fn demo_circuit() -> sim::Circuit {
    sim::Circuit {
        nodes: vec![
            node(1, "pin", -140.0, 80.0, 1),
            node(2, "pin", -140.0, -40.0, 1),
            node(3, "and", 0.0, 20.0, 0),
            node(4, "led", 140.0, 20.0, 0),
        ],
        wires: vec![
            sim::Wire {
                from: 1,
                from_port: 0,
                to: 3,
                to_port: 0,
            },
            sim::Wire {
                from: 2,
                from_port: 0,
                to: 3,
                to_port: 1,
            },
            sim::Wire {
                from: 3,
                from_port: 0,
                to: 4,
                to_port: 0,
            },
        ],
    }
}

fn node(id: u32, kind: &str, x: f32, y: f32, v: sim::Bit) -> sim::Node {
    sim::Node {
        id,
        kind: kind.into(),
        x,
        y,
        value: v,
        extra: Default::default(),
    }
}
