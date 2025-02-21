use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::WindowResolution;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rand::seq::SliceRandom;
use std::cmp::min;
use std::thread;
use std::time::Duration;
use std::vec;
use Algorithm::*;

const DEFAULT_SCREEN_RESOLUTION: (f32, f32) = (1200., 800.);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(
                    DEFAULT_SCREEN_RESOLUTION.0,
                    DEFAULT_SCREEN_RESOLUTION.1,
                ),
                title: "".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_systems(Update, settings_widget)
        .add_systems(Startup, setup)
        .add_systems(Update, render_list)
        .add_systems(Update, speed_controller)
        .add_systems(Update, insertion_sort)
        .add_systems(Update, selection_sort)
        .add_systems(Update, merge_sort)
        .add_systems(Update, schrödinger_sort)
        .add_systems(Update, end_animation)
        .insert_resource(SelectedAlgorithm(Insertion))
        .insert_resource(Operations(1))
        .insert_resource(MaxSpeed(Speed::Limited))
        .insert_resource(SpeedMode(SpeedLimit::Low))
        .insert_resource(List((1..=100).collect()))
        .insert_resource(NumberOfItems(100))
        .insert_resource(Delay(0))
        .insert_resource(SortingOngoing(false))
        .insert_resource(InsertionStep((0, 0)))
        .insert_resource(SelectionStep((0, 0, 0)))
        .insert_resource(MergeStep((1, 0, 0, 0, vec![], vec![])))
        .insert_resource(AnimationStep((0, 0, Insertion)))
        .insert_resource(Sort(false))
        .insert_resource(Paused(false))
        .insert_resource(Observed(true))
        .insert_resource(Sound(false))
        .run();
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Algorithm {
    Insertion,
    Selection,
    Merge,
    Schrödinger,
    Validation,
}

#[derive(Clone, Copy, PartialEq)]
enum Speed {
    Max,
    Limited,
}
#[derive(Clone, Copy, PartialEq)]
enum SpeedLimit {
    Low,
    High,
}

#[derive(Resource)]
struct SelectedAlgorithm(Algorithm);

#[derive(Resource)]
struct MaxSpeed(Speed);

#[derive(Resource)]
struct SpeedMode(SpeedLimit);

#[derive(Resource)]
struct Operations(u32);

#[derive(Resource)]
struct Delay(u64);

#[derive(Resource)]
struct List(Vec<i32>);

#[derive(Resource)]
struct NumberOfItems(i32);

#[derive(Resource)]
struct SortingOngoing(bool);

#[derive(Resource)]
struct Sort(bool);

#[derive(Resource)]
struct Paused(bool);

#[derive(Resource)]
struct InsertionStep((usize, usize)); //(index of main ptr, index of insertion ptr)

#[derive(Resource)]
struct SelectionStep((usize, usize, usize)); //(index of main ptr, index of selection ptr, index of selected value)

#[derive(Resource)]
struct MergeStep((usize, usize, usize, usize, Vec<i32>, Vec<i32>)); //(size of merge, merge number, ptr in vec1, ptr in vec2, vec1, vec2)

#[derive(Resource)]
struct AnimationStep((usize, u32, Algorithm)); //(index of main ptr, prev operations, prev selected algorithm)

#[derive(Resource)]
struct Observed(bool);

#[derive(Resource)]
struct PitchFrequency(f32);

#[derive(Resource)]
struct Sound(bool);

fn settings_widget(
    mut contexts: EguiContexts,
    mut selected: ResMut<SelectedAlgorithm>,
    mut max_speed: ResMut<MaxSpeed>,
    mut speed_limit: ResMut<SpeedMode>,
    mut operations: ResMut<Operations>,
    mut v: ResMut<List>,
    mut n: ResMut<NumberOfItems>,
    mut delay: ResMut<Delay>,
    mut sorting: ResMut<SortingOngoing>,
    mut observed: ResMut<Observed>,
    mut insertion_step: ResMut<InsertionStep>,
    mut selection_step: ResMut<SelectionStep>,
    mut merge_step: ResMut<MergeStep>,
    mut paused: ResMut<Paused>,
    mut sort: ResMut<Sort>,
    mut sound: ResMut<Sound>,
) {
    egui::Window::new("Controller").show(contexts.ctx_mut(), |ui| {
        if !sorting.0 {
            ui.add(egui::Slider::new(&mut n.0, 1..=1000).text("Number of items"));
            if n.0 as usize != v.0.len() {
                v.0 = (1..=n.0).collect();
            }
            if ui.button("Shuffle").clicked() {
                v.0.shuffle(&mut rand::thread_rng());
            }
        }
        ui.checkbox(&mut sound.0, "Sound");
        if selected.0 != Validation {
            ui.horizontal(|ui| {
                ui.radio_value(&mut max_speed.0, Speed::Limited, "Limit Speed");
                ui.radio_value(&mut max_speed.0, Speed::Max, "Max Speed");
            });
            if max_speed.0 == Speed::Max {
                ui.label(format!("Note: Max Speed is 100.000 Operations/Frame"));
            } else {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut speed_limit.0, SpeedLimit::Low, "Low Speed");
                    ui.radio_value(&mut speed_limit.0, SpeedLimit::High, "High Speed");
                });
                if speed_limit.0 == SpeedLimit::Low {
                    if operations.0 > 100 {
                        operations.0 = 100;
                    }
                    ui.add(egui::Slider::new(&mut operations.0, 0..=100).text("Operations/Frame"));
                    ui.add(egui::Slider::new(&mut delay.0, 0..=1000).text("Delay (ms)/Frame"));
                    ui.label(format!("Note: Delay will cause FPS to drop"));
                } else if speed_limit.0 == SpeedLimit::High {
                    delay.0 = 0;
                    ui.add(
                        egui::Slider::new(&mut operations.0, 0..=100000).text("Operations/Frame"),
                    );
                }
            }
        } else {
            ui.label(format!("Validating sort"));
        }

        if sorting.0 && !observed.0 && selected.0 == Schrödinger {
            ui.label(format!("Currently running: {:?}", selected.0));
            ui.label(format!(
                "Until the list is observed, it is both sorted and unsorted"
            ));
            if ui.button("Observe").clicked() {
                observed.0 = true;
            }
        }
        if paused.0 {
            if ui.button("Run 1 frame").clicked() {
                sort.0 = true;
            }
        }
        if !sorting.0 {
            egui::ComboBox::from_label("Sorting algorithm")
                .selected_text(format!("{:?}", selected.0))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected.0, Algorithm::Insertion, "Insertion");
                    ui.selectable_value(&mut selected.0, Algorithm::Selection, "Selection");
                    ui.selectable_value(&mut selected.0, Algorithm::Merge, "Merge");
                    ui.selectable_value(&mut selected.0, Algorithm::Schrödinger, "Schrödinger");
                });
            if ui.button("Start algorithm").clicked() {
                sorting.0 = true;
                paused.0 = false;
                if selected.0 == Schrödinger {
                    observed.0 = false;
                }
            }
        } else if observed.0 && selected.0 != Validation {
            ui.label(format!("Currently running: {:?}", selected.0));
            if selected.0 == Schrödinger {
                if ui.button("Stop Observing").clicked() {
                    observed.0 = false;
                }
            }
            ui.horizontal(|ui| {
                if paused.0 {
                    if ui.button("Continue algorithm").clicked() {
                        paused.0 = false;
                    }
                } else {
                    if ui.button("Pause algorithm").clicked() {
                        paused.0 = true;
                    }
                }
                if ui.button("Stop algorithm").clicked() {
                    sorting.0 = false;
                    paused.0 = false;
                    match selected.0 {
                        Algorithm::Insertion => insertion_step.0 = (0, 0),
                        Algorithm::Selection => selection_step.0 = (0, 0, 0),
                        Algorithm::Merge => merge_step.0 = (1, 0, 0, 0, vec![], vec![]),
                        _ => {}
                    }
                }
            });
        }
    });
}

fn speed_controller(
    mut operations: ResMut<Operations>,
    delay: Res<Delay>,
    sorting: ResMut<SortingOngoing>,
    mut sort: ResMut<Sort>,
    paused: Res<Paused>,
    max_speed: Res<MaxSpeed>,
    selected: Res<SelectedAlgorithm>,
) {
    if sorting.0 && !paused.0 {
        if selected.0 != Validation && max_speed.0 == Speed::Max {
            operations.0 = 100000;
            sort.0 = true;
        } else {
            thread::sleep(Duration::from_millis(delay.0));
            sort.0 = true;
        }
    }
}

fn insertion_sort(
    mut sort: ResMut<Sort>,
    mut step: ResMut<InsertionStep>,
    mut v: ResMut<List>,
    mut selected: ResMut<SelectedAlgorithm>,
    operations: Res<Operations>,
    mut end_step: ResMut<AnimationStep>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut frequency: ResMut<PitchFrequency>,
    mut commands: Commands,
    sound: Res<Sound>, //comment out to run insertion test
) {
    if sort.0 && selected.0 == Algorithm::Insertion {
        if operations.0 != 0 && sound.0 {
            //comment out to run insertion test
            frequency.0 = 200. + 1500.0 * step.0 .1.pow(3) as f32 / v.0.len().pow(3) as f32; //comment out to run insertion test
            commands.spawn(PitchBundle {
                //comment out to run insertion test
                source: pitch_assets.add(Pitch::new(frequency.0, Duration::from_millis(50))), //comment out to run insertion test
                settings: PlaybackSettings::DESPAWN, //comment out to run insertion test
            }); //comment out to run insertion test
        } //comment out to run insertion test
        for _ in 0..operations.0 {
            let v = &mut v.0;
            if step.0 .0 >= v.len() {
                sort.0 = false;
                step.0 = (0, 0);
                end_step.0 .2 = Insertion;
                selected.0 = Validation;
                break;
            } else {
                if step.0 .1 > 0 && v[step.0 .1 - 1] > v[step.0 .1] {
                    v.swap(step.0 .1 - 1, step.0 .1);
                    step.0 .1 -= 1;
                } else {
                    step.0 .0 += 1;
                    step.0 .1 = step.0 .0;
                }
            }
        }
        sort.0 = false;
    }
}

fn selection_sort(
    mut sort: ResMut<Sort>,
    mut step: ResMut<SelectionStep>,
    mut v: ResMut<List>,
    mut selected: ResMut<SelectedAlgorithm>,
    operations: Res<Operations>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut end_step: ResMut<AnimationStep>,
    mut frequency: ResMut<PitchFrequency>,
    mut commands: Commands,
    sound: Res<Sound>, //comment out to run selection test
) {
    if sort.0 && selected.0 == Algorithm::Selection {
        //comment out to run selection test
        if operations.0 != 0 && sound.0 {
            //comment out to run selection test
            frequency.0 = 200. + 1500.0 * step.0 .1.pow(3) as f32 / v.0.len().pow(3) as f32; //comment out to run selection test
            commands.spawn(PitchBundle {
                //comment out to run selection test
                source: pitch_assets.add(Pitch::new(frequency.0, Duration::from_millis(50))), //comment out to run selection test
                settings: PlaybackSettings::DESPAWN, //comment out to run selection test
            }); //comment out to run selection test
        } //comment out to run selection test
        for _ in 0..operations.0 {
            let v = &mut v.0;
            if step.0 .0 >= v.len() {
                sort.0 = false;
                step.0 = (0, 0, 0);
                end_step.0 .2 = Selection;
                selected.0 = Validation;
                break;
            } else {
                if step.0 .1 == step.0 .0 + 1 {
                    step.0 .2 = step.0 .0;
                }
                if step.0 .1 < v.len() {
                    if v[step.0 .1] < v[step.0 .2] {
                        step.0 .2 = step.0 .1;
                    }
                    step.0 .1 += 1;
                } else {
                    if step.0 .0 != step.0 .2 {
                        v.swap(step.0 .0, step.0 .2);
                    }
                    step.0 .0 += 1;
                    step.0 .1 = step.0 .0 + 1;
                }
            }
        }
        sort.0 = false;
    }
}

fn merge_sort(
    mut sort: ResMut<Sort>,
    mut step: ResMut<MergeStep>,
    mut v: ResMut<List>,
    mut selected: ResMut<SelectedAlgorithm>,
    operations: Res<Operations>,
    mut end_step: ResMut<AnimationStep>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut frequency: ResMut<PitchFrequency>,
    mut commands: Commands,
    sound: Res<Sound>, //comment out to run merge test
) {
    if sort.0 && selected.0 == Algorithm::Merge {
        //comment out to run merge test
        let left = step.0 .0 * step.0 .1 * 2; //comment out to run merge test
        let i1 = step.0 .2; //comment out to run merge test
        let i2 = step.0 .3; //comment out to run merge test
        let i = left + i1 + i2; //comment out to run merge test
        if operations.0 != 0 && sound.0 {
            //comment out to run merge test
            frequency.0 = 200. + 1500.0 * i.pow(3) as f32 / v.0.len().pow(3) as f32; //comment out to run merge test
            commands.spawn(PitchBundle {
                //comment out to run merge test
                source: pitch_assets.add(Pitch::new(frequency.0, Duration::from_millis(50))), //comment out to run merge test
                settings: PlaybackSettings::DESPAWN, //comment out to run merge test
            }); //comment out to run merge test
        } //comment out to run merge test
        let mut operation = 0;
        while operation < operations.0 {
            operation += 1;
            let v = &mut v.0;
            if step.0 .0 >= v.len() {
                sort.0 = false;
                step.0 = (1, 0, 0, 0, vec![], vec![]);
                end_step.0 .2 = Merge;
                selected.0 = Validation;
                break;
            } else {
                if step.0 .0 == 1 && step.0 .1 == 0 && step.0 .2 == 0 && step.0 .3 == 0 {
                    step.0 .4 = v[..step.0 .0].to_vec();
                    step.0 .5 = v[step.0 .0..step.0 .0 * 2].to_vec();
                }
                let left = step.0 .0 * step.0 .1 * 2;
                let i1 = step.0 .2;
                let v1 = &step.0 .4;
                let i2 = step.0 .3;
                let v2 = &step.0 .5;
                let i = left + i1 + i2;
                if i >= v.len() {
                    step.0 .0 *= 2;
                    if step.0 .0 >= v.len() {
                        continue;
                    }
                    (step.0 .1, step.0 .2, step.0 .3) = (0, 0, 0);
                    step.0 .4 = v[..step.0 .0].to_vec();
                    step.0 .5 = v[step.0 .0..min(step.0 .0 * 2, v.len())].to_vec();
                    continue;
                }
                if i1 < v1.len() {
                    if i2 < v2.len() {
                        if v1[i1] < v2[i2] {
                            v[i] = v1[i1];
                            step.0 .2 += 1;
                        } else {
                            v[i] = v2[i2];
                            step.0 .3 += 1;
                        }
                    } else {
                        v[i] = v1[i1];
                        step.0 .2 += 1;
                    }
                } else if i2 < v2.len() {
                    if i1 < v1.len() {
                        if v1[i1] < v2[i2] {
                            v[i] = v1[i1];
                            step.0 .2 += 1;
                        } else {
                            v[i] = v2[i2];
                            step.0 .3 += 1;
                        }
                    } else {
                        v[i] = v2[i2];
                        step.0 .3 += 1;
                    }
                } else {
                    operation -= 1;
                    step.0 .1 += 1;
                    (step.0 .2, step.0 .3) = (0, 0);
                    step.0 .4 = v[step.0 .0 * 2 * step.0 .1
                        ..min(step.0 .0 * 2 * step.0 .1 + step.0 .0, v.len())]
                        .to_vec();
                    if step.0 .0 * 2 * step.0 .1 + step.0 .0 < v.len() {
                        step.0 .5 = v[step.0 .0 * 2 * step.0 .1 + step.0 .0
                            ..min(step.0 .0 * 2 * step.0 .1 + step.0 .0 * 2, v.len())]
                            .to_vec();
                    } else {
                        step.0 .5 = vec![];
                    }
                }
            }
        }
        sort.0 = false;
    }
}

fn schrödinger_sort(
    mut sort: ResMut<Sort>,
    observed: Res<Observed>,
    mut v: ResMut<List>,
    mut selected: ResMut<SelectedAlgorithm>,
    operations: Res<Operations>,
    mut end_step: ResMut<AnimationStep>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut frequency: ResMut<PitchFrequency>,
    mut commands: Commands,
    sound: Res<Sound>,
) {
    if sort.0 && selected.0 == Algorithm::Schrödinger {
        if observed.0 {
            if operations.0 != 0 && sound.0 {
                frequency.0 = 200. + 1500.0 * v.0[0].pow(3) as f32 / v.0.len().pow(3) as f32;
                commands.spawn(PitchBundle {
                    source: pitch_assets.add(Pitch::new(frequency.0, Duration::from_millis(50))),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
        }
        for _ in 0..operations.0 {
            let mut sorted = true;
            for i in 1..v.0.len() {
                if v.0[i - 1] > v.0[i] {
                    sorted = false;
                }
            }
            if sorted {
                if observed.0 {
                    sort.0 = false;
                    end_step.0 .2 = Schrödinger;
                    selected.0 = Validation;
                    break;
                }
            } else {
                v.0.shuffle(&mut rand::thread_rng());
            }
        }
        sort.0 = false;
    }
}

fn end_animation(
    v: Res<List>,
    mut step: ResMut<AnimationStep>,
    mut operations: ResMut<Operations>,
    mut selected: ResMut<SelectedAlgorithm>,
    mut sort: ResMut<Sort>,
    mut sorting: ResMut<SortingOngoing>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut frequency: ResMut<PitchFrequency>,
    mut commands: Commands,
    sound: Res<Sound>,
) {
    if sort.0 && selected.0 == Algorithm::Validation {
        if sound.0 {
            frequency.0 = 200. + 1500.0 * step.0 .0.pow(3) as f32 / v.0.len().pow(3) as f32;
            commands.spawn(PitchBundle {
                source: pitch_assets.add(Pitch::new(frequency.0, Duration::from_millis(50))),
                settings: PlaybackSettings::DESPAWN,
            });
        }
        for _ in 0..operations.0 {
            if step.0 .0 == 0 {
                step.0 .1 = operations.0;
                operations.0 = 1 + (v.0.len() / 100) as u32;
                step.0 .0 += 1;
                break;
            } else if step.0 .0 >= v.0.len() - 1 {
                step.0 .0 = 0;
                operations.0 = step.0 .1 as u32;
                selected.0 = step.0 .2;
                sorting.0 = false;
                sort.0 = false;
                break;
            } else {
                step.0 .0 += 1;
            }
            sort.0 = false;
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(PitchFrequency(1000.0));
}

fn render_list(
    mut commands: Commands,
    v: ResMut<List>,
    sprites: Query<Entity, With<Sprite>>,
    windows: Query<&Window>,
    selected: Res<SelectedAlgorithm>,
    insertion_step: Res<InsertionStep>,
    selection_step: Res<SelectionStep>,
    merge_step: Res<MergeStep>,
    end_step: Res<AnimationStep>,
    ongoing: Res<SortingOngoing>,
    observed: Res<Observed>,
) {
    sprites.for_each(|entity| {
        commands.entity(entity).despawn();
    });
    let window = windows.single();
    let (window_width, window_height) = (window.width(), window.height());

    let len = v.0.len() as f32;

    for (n, i) in v.0.iter().zip(0..) {
        let mut color = Color::WHITE;
        if ongoing.0 {
            match selected.0 {
                Algorithm::Insertion => {
                    if i as usize == insertion_step.0 .1 {
                        color = Color::RED;
                    } else if i as usize <= insertion_step.0 .0 {
                        color = Color::GREEN;
                    }
                }
                Algorithm::Selection => {
                    if (i as usize) < selection_step.0 .0 {
                        color = Color::GREEN;
                    } else if i as usize == selection_step.0 .1 {
                        color = Color::RED;
                    } else if i as usize == selection_step.0 .2 {
                        color = Color::BLUE;
                    }
                }
                Algorithm::Merge => {
                    let left = merge_step.0 .0 * merge_step.0 .1 * 2;
                    if i >= left && i < left + merge_step.0 .2 + merge_step.0 .3 {
                        color = Color::GREEN;
                    }
                }
                Algorithm::Schrödinger => {
                    if !observed.0 {
                        break;
                    }
                }
                Algorithm::Validation => {
                    if i <= end_step.0 .0 {
                        color = Color::GREEN;
                    }
                }
            }
        }
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: color,
                custom_size: Some(Vec2::new(
                    0.9 * window_width / len,
                    (window_height - 200.) * *n as f32 / len,
                )),
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                -window_width / 2. + i as f32 * window_width / len + 0.05 * window_width / len,
                -window_height / 2.,
                0.,
            )),
            ..default()
        });
    }
}
//IMPORTANT
//to run these tests, it is unfortunately a requirement to comment out all the sound from all the algorithms as bevy wont create an event loop outside of the main thread
//all the lines that need to be commented out are marked by comments
#[test]
fn insertion_sort_test() {
    //checking if insertion sort is correct for one random vec for each len 1-1000
    use rand::{thread_rng, Rng};
    let mut app = App::new();
    app.add_systems(Update, insertion_sort);
    app.insert_resource(SelectedAlgorithm(Insertion));
    app.insert_resource(Operations(u32::MAX));
    app.insert_resource(InsertionStep((0, 0)));
    app.insert_resource(Sort(true));
    app.insert_resource(List((0..=100).collect()));
    app.insert_resource(AnimationStep((0, 0, Insertion)));

    for len in 1..=1000 {
        let mut step = app.world.resource_mut::<InsertionStep>();
        step.0 = (0, 0);
        let mut sort = app.world.resource_mut::<Sort>();
        sort.0 = true;
        let mut selected = app.world.resource_mut::<SelectedAlgorithm>();
        selected.0 = Insertion;
        let mut v = app.world.resource_mut::<List>();
        let random_vec: Vec<i32> = (0..len).map(|_| thread_rng().gen::<i32>()).collect();
        v.0 = random_vec;

        app.update();

        let v = app.world.resource::<List>();
        let mut sorted = true;
        for i in 1..v.0.len() {
            if v.0[i - 1] > v.0[i] {
                sorted = false;
            }
        }
        assert_eq!(sorted, true);
        assert_eq!(app.world.resource::<Sort>().0, false);
    }
}

#[test]
fn selection_sort_test() {
    //checking if selection sort is correct for one random vec for each len 1-1000
    use rand::{thread_rng, Rng};
    let mut app = App::new();

    app.add_systems(Update, selection_sort);
    app.insert_resource(SelectedAlgorithm(Selection));
    app.insert_resource(Operations(u32::MAX));
    app.insert_resource(SelectionStep((0, 0, 0)));
    app.insert_resource(Sort(true));
    app.insert_resource(SortingOngoing(true));
    app.insert_resource(List((0..=100).collect()));

    for len in 1..=1000 {
        let mut step = app.world.resource_mut::<SelectionStep>();
        step.0 = (0, 0, 0);
        let mut sort = app.world.resource_mut::<Sort>();
        sort.0 = true;
        let mut selected = app.world.resource_mut::<SelectedAlgorithm>();
        selected.0 = Selection;
        let mut v = app.world.resource_mut::<List>();
        let random_vec: Vec<i32> = (0..len).map(|_| thread_rng().gen::<i32>()).collect();
        v.0 = random_vec;

        app.update();

        let v = app.world.resource::<List>();
        let mut sorted = true;
        for i in 1..v.0.len() {
            if v.0[i - 1] > v.0[i] {
                sorted = false;
            }
        }

        assert_eq!(sorted, true);
        assert_eq!(app.world.resource::<Sort>().0, false);
        assert_eq!(app.world.resource::<SortingOngoing>().0, false);
    }
}

#[test]
fn merge_sort_test() {
    //checking if merge sort is correct for one random vec for each len 1-1000
    use rand::{thread_rng, Rng};
    let mut app = App::new();

    app.add_systems(Update, merge_sort);
    app.insert_resource(SelectedAlgorithm(Merge));
    app.insert_resource(Operations(u32::MAX));
    app.insert_resource(MergeStep((1, 0, 0, 0, vec![], vec![])));
    app.insert_resource(Sort(true));
    app.insert_resource(SortingOngoing(true));
    app.insert_resource(List((0..=100).collect()));

    for len in 1..=1000 {
        let mut step = app.world.resource_mut::<MergeStep>();
        step.0 = (1, 0, 0, 0, vec![], vec![]);
        let mut sort = app.world.resource_mut::<Sort>();
        sort.0 = true;
        let mut selected = app.world.resource_mut::<SelectedAlgorithm>();
        selected.0 = Merge;
        let mut v = app.world.resource_mut::<List>();
        let random_vec: Vec<i32> = (0..len).map(|_| thread_rng().gen::<i32>()).collect();
        v.0 = random_vec;

        app.update();

        let v = app.world.resource::<List>();

        let mut sorted = true;
        for i in 1..v.0.len() {
            if v.0[i - 1] > v.0[i] {
                sorted = false;
            }
        }

        assert_eq!(sorted, true);
        assert_eq!(app.world.resource::<Sort>().0, false);
        assert_eq!(app.world.resource::<SortingOngoing>().0, false);
    }
}
