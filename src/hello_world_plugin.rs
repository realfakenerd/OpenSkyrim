use bevy::app::{App, Update};
use bevy::prelude::*;
use crate::{Name, Person};

pub struct HelloPlugin;

#[derive(Resource)]
struct GreetTimer(Timer);

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (hello_world, (update_people, greet_people).chain()));
    }
}


fn add_people(mut commands:Commands) {
    commands.spawn((Person, Name("Links".to_string())));
    commands.spawn((Person, Name("Yoda".to_string())));
    commands.spawn((Person, Name("Luke".to_string())));
}


fn hello_world() {
    println!("Hello World");
}

fn greet_people(time: Res<Time>,mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("Hello {}", name.0);
        }
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Yoda" {
            name.0 = "Lucas".to_string();
            break;
        }
    }
}
