use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use rand::Rng;

const TICK_INTERVAL: f32 = 0.15;
const GAME_WIDTH: i64 = 12;
const GAME_HEIGHT: i64 = 12;
const BLOCK_SIZE: i64 = 32;
const GAP_SIZE: i64 = 2;
const FOOD_SIZE: i64 = 24;
const FOOD_INTERVAL: f32 = 1.00;
const FOOD_MAX_COUNT: usize = 4;

const SNAKE_COLOR: Color = Color::rgb(169.0 / 255.0, 224.0 / 255.0, 0.0 / 255.0);
const FOOD_COLOR: Color = Color::rgb(224.0 / 255.0, 45.0 / 255.0, 0.0 / 255.0);
const BG_COLOR: Color = Color::rgb(100.0 / 255.0, 157.0 / 255.0, 0.0 / 255.0);

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
    fn rev(&self) -> Dir {
        match self {
            Dir::Right => Dir::Left,
            Dir::Up => Dir::Down,
            Dir::Left => Dir::Right,
            Dir::Down => Dir::Up,
        }
    }
}

#[derive(Component, Clone, PartialEq)]
struct Pos {
    x: i64,
    y: i64,
}

impl Pos {
    fn random() -> Self {
        let x = rand::thread_rng().gen_range(0..GAME_WIDTH);
        let y = rand::thread_rng().gen_range(0..GAME_HEIGHT);
        Self { x, y }
    }

    fn move_dir(&mut self, dir: &Dir) {
        match dir {
            Dir::Right => self.x += 1,
            Dir::Up => self.y -= 1,
            Dir::Left => self.x -= 1,
            Dir::Down => self.y += 1,
        }
        if self.x < 0 {
            self.x = GAME_WIDTH - 1;
        }
        if self.x > GAME_WIDTH - 1 {
            self.x = 0;
        }
        if self.y < 0 {
            self.y = GAME_HEIGHT - 1;
        }
        if self.y > GAME_HEIGHT - 1 {
            self.y = 0;
        }
    }

    fn move_to(&mut self, target: &Pos) {
        self.x = target.x;
        self.y = target.y;
    }
}

struct Spr {}

impl Spr {
    fn new(size: i64, color: Color) -> SpriteBundle {
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(size as f32, size as f32)),
                anchor: bevy::sprite::Anchor::TopLeft,
                ..Default::default()
            },
            visibility: Visibility::Hidden,
            ..Default::default()
        }
    }

    fn translate(pos: &Pos, sprite: &Sprite, transform: &mut Transform) {
        transform.translation.x =
            (pos.x * (BLOCK_SIZE + GAP_SIZE) + GAP_SIZE) as f32
            - (GAME_WIDTH * (BLOCK_SIZE + GAP_SIZE) + GAP_SIZE) as f32 / 2.0;
        transform.translation.y =
            (-pos.y * (BLOCK_SIZE + GAP_SIZE) + GAP_SIZE) as f32
            + (GAME_HEIGHT * (BLOCK_SIZE + GAP_SIZE) - 3*GAP_SIZE) as f32 / 2.0;
        if let Some(size) = sprite.custom_size {
            transform.translation.x += (BLOCK_SIZE as f32 - size.x) / 2.0;
            transform.translation.y -= (BLOCK_SIZE as f32 - size.y) / 2.0;
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
#[derive(Component)]
struct Food {}

fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(BG_COLOR),
        },
        ..Default::default()
    });

    commands.spawn((
        Snake {
            dir: Dir::Right,
            next_dir: Dir::Right,
            lenght: 0,
        },
        Pos { x: 0, y: 0 },
        Spr::new(BLOCK_SIZE, SNAKE_COLOR),
    ));
}

fn main() {
    App::new()
    .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Snake".into(),
                    resolution: (
                        (GAME_WIDTH * (BLOCK_SIZE + GAP_SIZE) + GAP_SIZE) as f32,
                        (GAME_HEIGHT * (BLOCK_SIZE + GAP_SIZE) + GAP_SIZE) as f32
                    ).into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        ))
        .add_systems(Startup, init)
        .add_systems(First, input)
        .add_systems(PreUpdate, (tick, movement, translate_sprites))
        .add_systems(Update, (spawn_food, eat_food))
        .add_systems(PostUpdate, (log,))
        .insert_resource(TickTimer(Timer::from_seconds(
            TICK_INTERVAL,
            TimerMode::Repeating,
        )))
        .insert_resource(FoodTimer(Timer::from_seconds(
            FOOD_INTERVAL,
            TimerMode::Repeating,
        )))
        .run();
}

fn tick(time: Res<Time>, mut timer: ResMut<TickTimer>) {
    timer.0.tick(time.delta());
}

fn input(mut snakes: Query<&mut Snake>, input: Res<Input<KeyCode>>) {
    if let Some(dir) = match (
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
    } {
        for mut snake in snakes.iter_mut() {
            if dir != snake.dir.rev() {
                snake.next_dir = dir.clone();
            }
        }
    }
}

fn movement(
    mut commands: Commands,
    timer: Res<TickTimer>,
    mut snakes: Query<(&mut Snake, &mut Pos)>,
    mut segments: Query<(Entity, &mut Body, &mut Pos), Without<Snake>>,
) {
    if timer.0.just_finished() {
        for (_, mut segment, mut pos) in segments.iter_mut() {
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

            let mut dead = false;
            for (_, _, pos) in segments.iter_mut() {
                if head.clone() == pos.clone() {
                    dead = true;
                    break;
                }
            }
            if dead {
                snake.lenght = 0;
                for (entity, _, _) in segments.iter_mut() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn spawn_food(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<FoodTimer>,
    stuff: Query<&Pos>,
    food: Query<&Food>,
) {
    if timer.0.just_finished() && food.iter().len() < FOOD_MAX_COUNT {
        for _attempt in 0..5 {
            let pos = Pos::random();
            let mut collision = false;
            for p in stuff.iter() {
                if p == &pos {
                    collision = true;
                    break;
                }
            }
            if !collision {
                timer.0.tick(time.delta());
                commands.spawn((Food {}, pos, Spr::new(FOOD_SIZE, FOOD_COLOR)));
                break;
            }
        }
    } else {
        timer.0.tick(time.delta());
    }
}

fn eat_food(
    mut commands: Commands,
    mut snakes: Query<(&mut Snake, &mut Pos)>,
    mut segments: Query<&mut Body>,
    mut foods: Query<(Entity, &Food, &Pos), Without<Snake>>,
) {
    for (mut snake, head) in snakes.iter_mut() {
        for (food, _, food_pos) in foods.iter_mut() {
            if head.as_ref() == food_pos {
                for mut segment in segments.iter_mut() {
                    segment.move_countdown += 1
                }

                commands.entity(food).despawn();
                commands.spawn((
                    Body {
                        move_countdown: snake.lenght + 1,
                    },
                    head.clone(),
                    Spr::new(BLOCK_SIZE, SNAKE_COLOR),
                ));
                snake.lenght += 1;
            }
        }
    }
}

fn translate_sprites(
    timer: ResMut<TickTimer>,
    mut query: Query<(&Pos, &Sprite, &mut Transform, &mut Visibility), With<Sprite>>,
) {
    if timer.0.just_finished() {
        for (pos, sprite, mut transform, mut visibility) in query.iter_mut() {
            Spr::translate(pos, sprite, &mut transform);
            *visibility = Visibility::Visible;
        }
    }
}

fn log(timer: ResMut<TickTimer>, query: Query<&Pos, With<Snake>>) {
    if timer.0.just_finished() {
        for pos in query.iter() {
            println!("{:?} {:?}", pos.x, pos.y);
        }
    }
}
