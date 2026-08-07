use bevy::prelude::*;

use crate::components::{PlayButton, ProgressBarFill, StatusText};

pub fn ui_header() -> impl Scene {
    bsn! {
        #HeaderBanner
        Text::new("OPENSKYRIM")
        TextFont { font_size: FontSize::Px(36.0) }
        TextColor(Color::srgb(0.90, 0.80, 0.45))
    }
}

pub fn ui_drag_drop_zone() -> impl Scene {
    bsn! {
        #DragDropZone
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(70.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
        }
        BorderColor::all(Color::srgb(0.30, 0.35, 0.45))
        Children [
            (
                Text::new("Drag & Drop Mod Archives (.zip / .7z / .esp) Here")
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(Color::srgb(0.60, 0.65, 0.75))
            )
        ]
    }
}

pub fn ui_mod_manager_panel() -> impl Scene {
    bsn! {
        #ModManagerPanel
        Node {
            width: Val::Percent(92.0),
            height: Val::Percent(60.0),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(16.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
        }
        BackgroundColor(Color::srgb(0.12, 0.14, 0.18))
        BorderColor::all(Color::srgb(0.25, 0.28, 0.35))
        Children [
            (
                Text::new("Active Mod Load Order (0 Plugins Enabled)")
                TextFont { font_size: FontSize::Px(18.0) }
                TextColor(Color::srgb(0.85, 0.85, 0.85))
            ),
            ui_drag_drop_zone()
        ]
    }
}

pub fn ui_progress_bar() -> impl Scene {
    bsn! {
        #ProgressBarTrack
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(10.0),
            border: UiRect::all(Val::Px(1.0)),
            overflow: Overflow::clip(),
        }
        BackgroundColor(Color::srgb(0.15, 0.18, 0.22))
        BorderColor::all(Color::srgb(0.30, 0.35, 0.40))
        Children [
            (
                #ProgressBarFill
                ProgressBarFill
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                }
                BackgroundColor(Color::srgb(0.20, 0.70, 0.40))
            )
        ]
    }
}

pub fn ui_play_button() -> impl Scene {
    bsn! {
        #PlayButton
        PlayButton
        Button
        Node {
            width: Val::Px(240.0),
            height: Val::Px(48.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(Color::srgb(0.18, 0.55, 0.34))
        Children [
            (
                Text::new("PLAY OPENSKYRIM")
                TextFont { font_size: FontSize::Px(18.0) }
                TextColor(Color::WHITE)
            )
        ]
    }
}

pub fn ui_footer_controls() -> impl Scene {
    bsn! {
        #FooterControls
        Node {
            width: Val::Percent(92.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(12.0),
        }
        Children [
            (
                #StatusText
                StatusText
                Text::new("Locating Skyrim game files...")
                TextFont { font_size: FontSize::Px(15.0) }
                TextColor(Color::srgb(0.75, 0.80, 0.90))
            ),
            ui_progress_bar(),
            ui_play_button()
        ]
    }
}

pub fn launcher_scene_list() -> impl SceneList {
    bsn_list![
        Camera2d,
        (
            #RootWindow
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
            }
            BackgroundColor(Color::srgb(0.08, 0.09, 0.12))
            Children [
                ui_header(),
                ui_mod_manager_panel(),
                ui_footer_controls()
            ]
        )
    ]
}

pub fn setup_ui(mut commands: Commands) {
    commands.spawn_scene_list(launcher_scene_list());
}
