use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{timing::Time, transform::Transform},
    ecs::{Component, DenseVecStorage, Entity},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{Anchor, LineMode, TtfFormat, UiText, UiTransform},
};

use crate::audio::initialize_audio;

pub const ARENA_HEIGHT: f32 = 100.0;
pub const ARENA_WIDTH: f32 = 100.0;

pub const PADDLE_HEIGHT: f32 = 16.0;
pub const PADDLE_WIDTH: f32 = 4.0;

pub const BALL_VELOCITY_X: f32 = 75.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_RADIUS: f32 = 2.0;

#[derive(Default)]
pub struct Pong {
    ball_spawn_timer: Option<f32>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Wait two seconds before spawning the ball
        self.ball_spawn_timer.replace(2.0);

        // Load the spritesheet necessary to render the graphics.
        // `spritesheet` is the layout of the sprites on the image;
        // `texture` is the pixel data.
        self.sprite_sheet_handle
            .replace(init::load_sprite_sheet(world));

        init::paddles(world, self.sprite_sheet_handle.clone().unwrap());
        init::scoreboard(world);
        init::camera(world);
        initialize_audio(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(mut timer) = self.ball_spawn_timer.take() {
            // If the timer isn't expired yet, subtract the time that passed since the last update.
            {
                let time = data.world.fetch::<Time>();
                timer -= time.delta_seconds();
            }

            if timer <= 0.0 {
                // When timer has expired, spawn the ball
                init::ball(data.world, self.sprite_sheet_handle.clone().unwrap());
            } else {
                // If timer hasn't expired, put it back into state
                self.ball_spawn_timer.replace(timer);
            }
        }

        Trans::None
    }
}

mod init {
    use super::*;

    pub fn camera(world: &mut World) {
        let mut transform = Transform::default();
        transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

        world
            .create_entity()
            .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
            .with(transform)
            .build();
    }

    pub fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "texture/pong_spritesheet.png",
                ImageFormat::default(),
                (),
                &texture_storage,
            )
        };

        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load(
            "texture/pong_spritesheet.ron", // Here we load the associated ron file
            SpriteSheetFormat(texture_handle),
            (),
            &sprite_sheet_store,
        )
    }

    pub fn paddles(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
        let sprite_render = SpriteRender::new(sprite_sheet_handle, 0);
        let mut left_transform = Transform::default();
        let mut right_transform = Transform::default();

        let y = ARENA_HEIGHT / 2.0;
        left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
        right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

        world
            .create_entity()
            .with(sprite_render.clone())
            .with(Paddle::new(Side::Left))
            .with(left_transform)
            .build();

        world
            .create_entity()
            .with(sprite_render)
            .with(Paddle::new(Side::Right))
            .with(right_transform)
            .build();
    }

    pub fn ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
        // Create a translation object
        let mut local_transform = Transform::default();
        local_transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

        // Assign the sprite fro the ball. The ball is the second sprite in the sheet
        let sprite_render = SpriteRender::new(sprite_sheet_handle, 1);

        world
            .create_entity()
            .with(sprite_render)
            .with(Ball {
                radius: BALL_RADIUS,
                velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
            })
            .with(local_transform)
            .build();
    }

    pub fn scoreboard(world: &mut World) {
        let font = world.read_resource::<Loader>().load(
            "font/square.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        );

        let p1_transform = UiTransform::new(
            "P1".to_string(),
            Anchor::TopMiddle,
            Anchor::TopMiddle,
            -50.,
            -50.,
            1.,
            200.,
            50.,
        );

        let p2_transform = UiTransform::new(
            "P2".to_string(),
            Anchor::TopMiddle,
            Anchor::TopMiddle,
            50.,
            -50.,
            1.,
            200.,
            50.,
        );

        let p1_score = world
            .create_entity()
            .with(p1_transform)
            .with(UiText::new(
                font.clone(),
                "0".to_string(),
                [1., 1., 1., 1.],
                50.,
                LineMode::Single,
                Anchor::Middle,
            ))
            .build();

        let p2_score = world
            .create_entity()
            .with(p2_transform)
            .with(UiText::new(
                font.clone(),
                "0".to_string(),
                [1., 1., 1., 1.],
                50.,
                LineMode::Single,
                Anchor::Middle,
            ))
            .build();

        world.insert(ScoreText { p1_score, p2_score })
    }
}

#[derive(PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    fn new(side: Side) -> Paddle {
        Paddle {
            side,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

/// ScoreBoard contains the actual score data
#[derive(Default)]
pub struct ScoreBoard {
    pub score_left: i32,
    pub score_right: i32,
}

/// ScoreText contains the ui text components that display the score
pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}
