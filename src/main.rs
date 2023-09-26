use bevy::prelude::*;

#[derive(Resource)]
struct TickTimer(Timer);

#[derive(Resource)]
struct InputTimer(Timer);

#[derive(Resource)]
struct FoodTimer(Timer);

#[derive(Clone)]
enum Dir {
    Right,
    Up,
    Left,
    Down,
}

#[derive(Component)]
struct Pos {
    x: i64,
    y: i64,
}

impl Pos {
    fn mv(&mut self, dir: &Dir) {
        match dir {
            Dir::Right => self.x += 1,
            Dir::Up => self.y -= 1,
            Dir::Left => self.x -= 1,
            Dir::Down => self.y += 1,
        }
    }

    fn moveto(&mut self, to: &Pos) {
        self.x = to.x;
        self.y = to.y;
    }

    fn translate(&self, transform: &mut Transform) {
        transform.translation.x = self.x as f32 * 64.0;
        transform.translation.y = -self.y as f32 * 64.0;
    }
}

#[derive(Component)]
struct Snake {
    dir: Dir,
    lenght: u64,
}

#[derive(Component)]
struct Body {
    move_countdown: u64,
}

#[derive(Component)]
struct Food {}

fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let pos = Pos { x: 0, y: 0 };
    let mut transform = Transform {
        translation: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        ..Default::default()
    };
    pos.translate(&mut transform);
    commands.spawn((
        Snake {
            dir: Dir::Right,
            lenght: 0,
        },
        pos,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 1.0, 0.5),
                custom_size: Some(Vec2::new(64.0, 64.0)),
                anchor: Default::default(),
                ..Default::default()
            },
            transform,
            ..Default::default()
        },
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, init)
        .add_systems(PreUpdate, tick)
        .add_systems(Update, (input, movement))
        .add_systems(PostUpdate, (grow, sprite_transforms, log))
        .insert_resource(TickTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .insert_resource(FoodTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
        .insert_resource(InputTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .run();
}

fn log(timer: ResMut<TickTimer>, query: Query<&Pos>) {
    if timer.0.just_finished() {
        for pos in query.iter() {
            println!("{:?} {:?}", pos.x, pos.y);
        }
    }
}

fn sprite_transforms(timer: ResMut<TickTimer>, mut query: Query<(&Pos, &mut Transform)>) {
    if timer.0.just_finished() {
        for (pos, mut transform) in query.iter_mut() {
            pos.translate(&mut transform)
        }
    }
}

fn input(
    time: Res<Time>,
    mut timer: ResMut<InputTimer>,
    mut snakes: Query<&mut Snake>,
    input: Res<Input<KeyCode>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let dir = match (
            input.pressed(KeyCode::Up),
            input.pressed(KeyCode::Down),
            input.pressed(KeyCode::Left),
            input.pressed(KeyCode::Right),
        ) {
            (true, false, _, _) => Some(Dir::Up),
            (false, true, _, _) => Some(Dir::Down),
            (_, _, true, false) => Some(Dir::Left),
            (_, _, false, true) => Some(Dir::Right),
            _ => None,
        };

        if let Some(d) = dir {
            for mut snake in snakes.iter_mut() {
                snake.dir = d.clone();
            }
        }
    }
}

fn tick(time: Res<Time>, mut timer: ResMut<TickTimer>) {
    timer.0.tick(time.delta());
}

fn movement(
    timer: Res<TickTimer>,
    mut snakes: Query<(&mut Snake, &mut Pos)>,
    mut parts: Query<(&mut Body, &mut Pos), Without<Snake>>,
) {
    if timer.0.just_finished() {
        for (mut part, mut pos) in parts.iter_mut() {
            if part.move_countdown > 0 {
                part.move_countdown -= 1;
            } else {
                for (snake, spos) in snakes.iter_mut() {
                    part.move_countdown = snake.lenght - 1;
                    pos.moveto(&spos);
                }
            }
        }
        for (snake, mut pos) in snakes.iter_mut() {
            pos.mv(&snake.dir);
        }
    }
}

fn grow(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<FoodTimer>,
    mut snakes: Query<(&mut Snake, &mut Pos)>,
    mut parts: Query<&mut Body>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut part in parts.iter_mut() {
            part.move_countdown += 1
        }

        for (mut snake, pos) in snakes.iter_mut() {
            let pos = Pos { x: pos.x, y: pos.y };
            let mut transform = Transform::default();
            pos.translate(&mut transform);

            commands.spawn((
                Body {
                    move_countdown: snake.lenght + 1,
                },
                pos,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.5, 1.0, 0.5),
                        custom_size: Some(Vec2::new(64.0, 64.0)),
                        ..Default::default()
                    },
                    transform,
                    ..Default::default()
                },
            ));
            snake.lenght += 1;
        }
    }
}
