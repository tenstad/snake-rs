use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

const BLOCK_SIZE: i64 = 32;
const GAP_SIZE: i64 = 2;
const TICK_INTERVAL: f32 = 0.15;
const GROW_INTERVAL: f32 = 0.8;

#[derive(Resource)]
struct TickTimer(Timer);

#[derive(Resource)]
struct FoodTimer(Timer);

#[derive(Clone, PartialEq)]
enum Dir {
    Right,
    Up,
    Left,
    Down,
}

impl Dir {
    fn opposite(&self) -> Dir {
        match self {
            Dir::Right => Dir::Left,
            Dir::Up => Dir::Down,
            Dir::Left => Dir::Right,
            Dir::Down => Dir::Up,
        }
    }
}

#[derive(Component)]
struct Pos {
    x: i64,
    y: i64,
}

impl Pos {
    fn move_dir(&mut self, dir: &Dir) {
        match dir {
            Dir::Right => self.x += 1,
            Dir::Up => self.y -= 1,
            Dir::Left => self.x -= 1,
            Dir::Down => self.y += 1,
        }
    }

    fn move_to(&mut self, target: &Pos) {
        self.x = target.x;
        self.y = target.y;
    }

    fn translate(&self, transform: &mut Transform) {
        transform.translation.x = (self.x * (BLOCK_SIZE + GAP_SIZE)) as f32;
        transform.translation.y = (-self.y * (BLOCK_SIZE + GAP_SIZE)) as f32;
    }

    fn sprite(&self) -> SpriteBundle {
        let mut transform = Transform::default();
        self.translate(&mut transform);
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(BLOCK_SIZE as f32, BLOCK_SIZE as f32)),
                ..Default::default()
            },
            transform,
            ..Default::default()
        }
    }
}

#[derive(Component)]
struct Snake {
    dir: Dir,
    next_dir: Dir,
    lenght: u64,
}

#[derive(Component)]
struct Body {
    move_countdown: u64,
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::rgb(169.0 / 255.0, 224.0 / 255.0, 0.0)),
        },
        ..Default::default()
    });

    let pos = Pos { x: 0, y: 0 };
    commands.spawn((
        Snake {
            dir: Dir::Right,
            next_dir: Dir::Right,
            lenght: 0,
        },
        pos.sprite(),
        pos,
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, init)
        .add_systems(PreUpdate, tick)
        .add_systems(Update, (input, movement))
        .add_systems(PostUpdate, (grow, translate_sprites, log))
        .insert_resource(TickTimer(Timer::from_seconds(
            TICK_INTERVAL,
            TimerMode::Repeating,
        )))
        .insert_resource(FoodTimer(Timer::from_seconds(
            GROW_INTERVAL,
            TimerMode::Repeating,
        )))
        .run();
}

fn tick(time: Res<Time>, mut timer: ResMut<TickTimer>) {
    timer.0.tick(time.delta());
}

fn input(mut snakes: Query<&mut Snake>, input: Res<Input<KeyCode>>) {
    let dir = match (
        input.pressed(KeyCode::Up) || input.pressed(KeyCode::W),
        input.pressed(KeyCode::Down) || input.pressed(KeyCode::S),
        input.pressed(KeyCode::Left) || input.pressed(KeyCode::A),
        input.pressed(KeyCode::Right) || input.pressed(KeyCode::D),
    ) {
        (true, false, _, _) => Some(Dir::Up),
        (false, true, _, _) => Some(Dir::Down),
        (_, _, true, false) => Some(Dir::Left),
        (_, _, false, true) => Some(Dir::Right),
        _ => None,
    };

    if let Some(d) = dir {
        for mut snake in snakes.iter_mut() {
            if d != snake.dir.opposite() {
                snake.next_dir = d.clone();
            }
        }
    }
}

fn movement(
    timer: Res<TickTimer>,
    mut snakes: Query<(&mut Snake, &mut Pos)>,
    mut segments: Query<(&mut Body, &mut Pos), Without<Snake>>,
) {
    if timer.0.just_finished() {
        for (mut segment, mut pos) in segments.iter_mut() {
            if segment.move_countdown > 0 {
                segment.move_countdown -= 1;
            } else {
                for (snake, head) in snakes.iter_mut() {
                    pos.move_to(&head);
                    segment.move_countdown = snake.lenght - 1;
                }
            }
        }
        for (mut snake, mut head) in snakes.iter_mut() {
            head.move_dir(&snake.next_dir);
            snake.dir = snake.next_dir.clone();
        }
    }
}

fn grow(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<FoodTimer>,
    mut snakes: Query<(&mut Snake, &mut Pos)>,
    mut segments: Query<&mut Body>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut segment in segments.iter_mut() {
            segment.move_countdown += 1
        }

        for (mut snake, head) in snakes.iter_mut() {
            let pos = Pos {
                x: head.x,
                y: head.y,
            };
            commands.spawn((
                Body {
                    move_countdown: snake.lenght + 1,
                },
                pos.sprite(),
                pos,
            ));
            snake.lenght += 1;
        }
    }
}

fn translate_sprites(timer: ResMut<TickTimer>, mut query: Query<(&Pos, &mut Transform)>) {
    if timer.0.just_finished() {
        for (pos, mut transform) in query.iter_mut() {
            pos.translate(&mut transform)
        }
    }
}

fn log(timer: ResMut<TickTimer>, query: Query<&Pos>) {
    if timer.0.just_finished() {
        for pos in query.iter() {
            println!("{:?} {:?}", pos.x, pos.y);
        }
    }
}
