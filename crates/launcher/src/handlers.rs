use crate::{
    components::{PlayButton, ProgressBarFill, StatusText},
    game_detection::find_skyrim_data_dir,
};

use super::{
    ConversionChannel, ConversionProgressEvent, ConversionStatus, GamePathConfig, LauncherState,
};
use bevy::prelude::*;
use converter::{
    AssetPipeline, PipelineConfig, ProgressEvent, ProgressStage, cache::ConversionManifest,
};
use crossbeam_channel::{Sender, unbounded};
use std::thread;

pub fn detect_skyrim_and_start_conversion(
    mut commands: Commands,
    mut next_state: ResMut<NextState<LauncherState>>,
    mut config: ResMut<GamePathConfig>,
    mut status: ResMut<ConversionStatus>,
) {
    let output_dir = config.converted_assets_path.clone();
    let conversion_complete = ConversionManifest::load(
        &output_dir.join("conversion-manifest.json"),
    )
    .is_ok_and(|manifest| manifest.complete && output_dir.join("skyrim_world.db").is_file());

    if let Some(data_dir) = find_skyrim_data_dir() {
        config.skyrim_data_path = Some(data_dir.clone());

        if conversion_complete {
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

            let output_dir = config.converted_assets_path.clone();
            thread::spawn(move || {
                run_background_converter(tx, data_dir, output_dir);
            });
        }
        next_state.set(LauncherState::ModManager);
    } else {
        status.current_step =
            "Skyrim Special Edition was not found in Steam libraries or local folders.".into();
        next_state.set(LauncherState::FirstRunSetup);
    }
}

fn run_background_converter(
    tx: Sender<ConversionProgressEvent>,
    data_dir: std::path::PathBuf,
    output_dir: std::path::PathBuf,
) {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = tx.send(ConversionProgressEvent {
                percentage: 0.0,
                current_file: format!("Failed to start converter: {error}"),
                finished: false,
                failed: true,
            });
            return;
        }
    };
    runtime.block_on(async move {
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<ProgressEvent>(256);
        let ui_tx = tx.clone();
        let forwarder = tokio::spawn(async move {
            while let Some(event) = progress_rx.recv().await {
                let _ = ui_tx.send(to_launcher_event(event));
            }
        });
        let result =
            AssetPipeline::run_async(PipelineConfig::new(data_dir, output_dir), progress_tx).await;
        let _ = forwarder.await;
        match result {
            Ok(report) => {
                let (message, percentage) = if report.complete {
                    (
                        format!(
                            "Converted {} assets ({} reused)",
                            report.converted, report.cache_hits
                        ),
                        1.0,
                    )
                } else {
                    (
                        format!(
                            "Conversion incomplete: {} input(s) skipped; see conversion-manifest.json",
                            report.skipped
                        ),
                        0.99,
                    )
                };
                let _ = tx.send(ConversionProgressEvent {
                    percentage,
                    current_file: message,
                    finished: report.complete,
                    failed: !report.complete,
                });
            }
            Err(error) => {
                let _ = tx.send(ConversionProgressEvent {
                    percentage: 0.0,
                    current_file: format!("Asset conversion failed: {error:#}"),
                    finished: false,
                    failed: true,
                });
            }
        }
    });
}

fn to_launcher_event(event: ProgressEvent) -> ConversionProgressEvent {
    let stage_base = match event.stage {
        ProgressStage::Discovering => 0.0,
        ProgressStage::Extracting => 0.05,
        ProgressStage::Database => 0.30,
        ProgressStage::Textures => 0.50,
        ProgressStage::Meshes => 0.70,
        ProgressStage::Scripts => 0.85,
        ProgressStage::Validating => 0.93,
        ProgressStage::Publishing => 0.97,
        ProgressStage::Complete => 1.0,
    };
    let stage_width = match event.stage {
        ProgressStage::Extracting => 0.25,
        ProgressStage::Database => 0.20,
        ProgressStage::Textures => 0.20,
        ProgressStage::Meshes => 0.15,
        ProgressStage::Scripts => 0.08,
        ProgressStage::Validating => 0.04,
        ProgressStage::Publishing => 0.03,
        _ => 0.0,
    };
    ConversionProgressEvent {
        percentage: (stage_base + event.fraction() * stage_width).clamp(0.0, 1.0),
        current_file: event.current_file.map_or(event.message.clone(), |path| {
            format!("{}: {}", event.message, path.display())
        }),
        finished: event.stage == ProgressStage::Complete,
        failed: false,
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
            status.has_failed = event.failed;

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
