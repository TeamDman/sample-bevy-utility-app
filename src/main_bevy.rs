use eyre::Result;

pub fn main_bevy() -> Result<()> {
    use bevy::prelude::*;

    fn setup(mut commands: Commands) {
        commands.spawn(Camera2d);
        commands.spawn(Text::new("Hello Bevy!"));
    }

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, |mouse: Res<ButtonInput<MouseButton>>| {
            if mouse.just_pressed(MouseButton::Left) {
                info!("Left mouse button pressed");
            }
        })
        .run();

    Ok(())
}
