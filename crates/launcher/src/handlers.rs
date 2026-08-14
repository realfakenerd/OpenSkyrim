use crate::{
    components::{PlayButton, ProgressBarFill, StatusText},
    game_detection::find_skyrim_data_dir,
};

use super::{
    ConversionChannel, ConversionProgressEvent, ConversionStatus, GamePathConfig, LauncherState,
};
use bevy::prelude::*;
use crossbeam_channel::{Sender, unbounded};
use std::thread;

pub fn detect_skyrim_and_start_conversion(
    mut commands: Commands,
    mut next_state: ResMut<NextState<LauncherState>>,
    mut config: ResMut<GamePathConfig>,
    mut status: ResMut<ConversionStatus>,
) {
    let db_exists = std::path::Path::new("./modern_assets/skyrim_world.db").exists();

    if let Some(data_dir) = find_skyrim_data_dir() {
        config.skyrim_data_path = Some(data_dir.clone());

        if db_exists {
            status.progress = 1.0;
            status.current_step = format!(
                "Skyrim found at {}. Assets converted! Ready to play.",
                data_dir.display()
            );
            status.is_complete = true;
        } else {
            status.current_step = format!(
                "Skyrim found at {}. Starting conversion...",
                data_dir.display()
            );
            let (tx, rx) = unbounded::<ConversionProgressEvent>();
            commands.insert_resource(ConversionChannel { receiver: rx });

            thread::spawn(move || {
                run_background_converter(tx);
            });
        }
        next_state.set(LauncherState::ModManager);
    } else {
        status.current_step =
            "Skyrim Special Edition was not found in Steam libraries or local folders.".into();
        next_state.set(LauncherState::FirstRunSetup);
    }
}

fn run_background_converter(tx: Sender<ConversionProgressEvent>) {
    let steps = [
        "Unpacking BSA archives (Skyrim - Textures0.bsa)...",
        "Converting textures (.dds -> KTX2 Basis Universal)...",
        "Converting 3D meshes (.nif -> glTF 2.0)...",
        "Parsing Skyrim.esm -> skyrim_world.db (SQLite)...",
        "Transpiling Papyrus scripts (.pex -> Luau)...",
    ];

    for (i, step) in steps.iter().enumerate() {
        let percentage = (i as f32 + 1.0) / steps.len() as f32;
        let is_last = i == steps.len() - 1;

        let _ = tx.send(ConversionProgressEvent {
            percentage,
            current_file: (*step).to_string(),
            finished: is_last,
        });

        thread::sleep(std::time::Duration::from_millis(600));
    }
}

pub fn update_conversion_progress(
    channel: Option<Res<ConversionChannel>>,
    mut status: ResMut<ConversionStatus>,
    mut fill_query: Query<&mut Node, With<ProgressBarFill>>,
    mut text_query: Query<&mut Text, With<StatusText>>,
) {
    if let Some(chan) = channel {
        while let Ok(event) = chan.receiver.try_recv() {
            status.progress = event.percentage;
            status.current_step = event.current_file.clone();
            status.is_complete = event.finished;

            for mut node in fill_query.iter_mut() {
                node.width = Val::Percent(event.percentage * 100.0);
            }

            for mut text in text_query.iter_mut() {
                text.0 = format!("{} ({:.0}%)", event.current_file, event.percentage * 100.0);
            }
        }
    }
}

pub fn sync_status_text(
    status: Res<ConversionStatus>,
    mut text_query: Query<&mut Text, With<StatusText>>,
) {
    if !status.is_changed() {
        return;
    }

    for mut text in text_query.iter_mut() {
        text.0 = status.current_step.clone();
    }
}

pub fn handle_play_button_click(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    status: Res<ConversionStatus>,
    mut next_state: ResMut<NextState<LauncherState>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if status.is_complete {
                println!("Launching OpenSkyrim Engine binary...");
                next_state.set(LauncherState::LaunchingEngine);
            } else {
                println!(
                    "Conversion in progress ({:.0}%)... Please wait.",
                    status.progress * 100.0
                );
            }
        }
    }
}

pub fn handle_mod_drag_and_drop(mut dnd_events: MessageReader<FileDragAndDrop>) {
    for event in dnd_events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            println!("Mod dropped into launcher: {:?}", path_buf);
        }
    }
}
