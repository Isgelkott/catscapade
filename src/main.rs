use std::{f32::consts::PI, sync::LazyLock};

use asefile::AsepriteFile;
use image::*;
use macroquad::{prelude::*, rand::rand};
fn load_ase_texture(bytes: &[u8], layer: Option<u32>, frame: Option<u32>) -> Texture2D {
    let img = AsepriteFile::read(bytes).unwrap();
    let frame = frame.unwrap_or(0);
    let img = if let Some(layer) = layer {
        img.layer(layer).frame(frame).image()
    } else {
        img.frame(0).image()
    };

    let new = Image {
        width: img.width() as u16,
        height: img.height() as u16,
        bytes: img.as_bytes().to_vec(),
    };
    let texture = Texture2D::from_image(&new);
    texture.set_filter(FilterMode::Nearest);
    texture
}
fn create_camera(dimensions: Vec2) -> Camera2D {
    let rt = render_target(dimensions.y as u32, dimensions.y as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    Camera2D {
        render_target: Some(rt),
        zoom: Vec2::new(1.0 / dimensions.x * 2.0, 1.0 / dimensions.y * 2.0),
        target: vec2((dimensions.x / 2.0).floor(), (dimensions.y / 2.0).floor()),
        ..Default::default()
    }
}
pub fn load_animation_from_tag(bytes: &[u8], tag: &str) -> (Vec<(Texture2D, u32)>, u32) {
    let file = AsepriteFile::read(bytes).unwrap();
    dbg!(tag);
    let tag = file.tag_by_name(tag).unwrap();
    let start = tag.from_frame();
    let end = tag.to_frame();
    let mut frames = Vec::new();
    let mut duration = 0;
    for frame in start..=end {
        let img = file.frame(frame);
        let time = img.duration();
        duration += time;
        let img = img.image();
        let texture = Texture2D::from_image(&Image {
            width: img.width() as u16,
            height: img.height() as u16,
            bytes: img.as_bytes().to_vec(),
        });
        texture.set_filter(FilterMode::Nearest);
        frames.push((texture, time));
    }
    (frames, duration)
}
struct Spritesheet {
    texture: Texture2D,
    widht: f32,
    height: f32,
}
impl Spritesheet {
    fn draw_from(&self, world_pos: Vec2, texture_coord: (u8, u8), scale: f32) {
        draw_texture_ex(
            &self.texture,
            world_pos.x,
            world_pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect {
                    x: texture_coord.0 as f32 * self.widht,
                    y: texture_coord.1 as f32 * self.height,
                    w: self.widht,
                    h: self.height,
                }),
                dest_size: Some(vec2(self.widht, self.height) * scale),
                ..Default::default()
            },
        )
    }
}
type Animation = (Vec<(Texture2D, u32)>, u32);
struct PlayerAnimations {
    walk: Animation,
}
const CAT_SPEED: f32 = 70.0;
struct Cat {
    pos: Vec2,
    size: Vec2,
    direction: Vec2,
    animations: PlayerAnimations,
}
impl Cat {
    fn new() -> Self {
        let animations = PlayerAnimations {
            walk: load_animation_from_tag(include_bytes!("../assets/cat.ase"), "walk"),
        };

        Self {
            pos: vec2(100.0, 200.0),
            size: vec2(
                animations.walk.0[0].0.width(),
                animations.walk.0[0].0.height(),
            ),
            direction: Vec2::ZERO,
            animations,
        }
    }
    fn update(&mut self, map: &Map) {
        let mut animation = &self.animations.walk;
        let mut direction = Vec2::ZERO;
        if is_key_down(KeyCode::A) {
            direction.x += -1.0;
            animation = &self.animations.walk;
        }
        if is_key_down(KeyCode::D) {
            direction.x += 1.0;
            animation = &self.animations.walk;
        }
        if is_key_down(KeyCode::S) {
            direction.y += 1.0;
            animation = &self.animations.walk;
        }
        if is_key_down(KeyCode::W) {
            direction.y += -1.0;
            animation = &self.animations.walk;
        }
        let rotation = 0.5 * PI + direction.y.atan2(direction.x);
        self.direction += direction.normalize_or_zero() * CAT_SPEED * get_frame_time();
        let collision_points = [
            (0.0, 0.0),
            (self.size.x, 0.0),
            (0.0, self.size.y),
            (self.size.x, self.size.y),
        ];

        for (index, p) in collision_points.iter().enumerate() {
            let map_pos = (self.pos + self.direction + vec2(p.0, p.1)) / (16.0 * MAP_SCALE_FACTOR);
            let pottential_collider =
                &map.tiles[map_pos.y as usize * map.width as usize + map_pos.x as usize];

            if pottential_collider.collision && !is_key_down(KeyCode::Space) {
                println!("collid with {:?}, with:{index}", pottential_collider);
                let x0 = map_pos.x.floor() * 16.0 * MAP_SCALE_FACTOR - p.0;
                let x1 = (map_pos.x.floor() + 1.0) * MAP_SCALE_FACTOR * 16.0 - p.0;
                let y0 = map_pos.y.floor() * 16.0 * MAP_SCALE_FACTOR - p.1;
                let y1 = map_pos.y.ceil() * MAP_SCALE_FACTOR * 16.0 - p.1;
                self.pos.x = self.pos.x.clamp(x0, x1);
                self.pos.y = self.pos.y.clamp(y0, y1);
                if self.pos.x == x0 || self.pos.x == x1 {
                    self.direction.x = 0.0;
                } else if self.pos.y == y0 || self.pos.y == y1 {
                    self.direction.y = 0.0;
                }
                break;
            }
        }

        self.pos += self.direction;

        self.direction = self.direction.lerp(Vec2::ZERO, 0.3);
        if self.direction.x.abs() < 0.3 && self.direction.y.abs() < 0.3 {
            self.direction = Vec2::ZERO;
        }
        if is_key_down(KeyCode::F) {
            dbg!(self.pos, self.direction);
        }
        let mut time = (get_time() * 1000.0) % animation.1 as f64;
        for i in &animation.0 {
            if time <= i.1 as f64 {
                draw_texture_ex(
                    &i.0,
                    self.pos.x,
                    self.pos.y,
                    WHITE,
                    DrawTextureParams {
                        rotation,
                        ..Default::default()
                    },
                );
                break;
            } else {
                time -= i.1 as f64;
            }
        }
    }
}
struct Mouse<'a> {
    is_rainbow: bool,
    speed: f32,
    scare_clock: f32,
    pos: Vec2,
    size: Vec2,
    direction: Vec2,
    animation: &'a Animation,
}
const SCREEN_SIZE: Vec2 = Vec2 { x: 160.0, y: 160.0 };
#[derive(Debug, PartialEq)]

enum Layer {
    Floor,
    Decor,
    Collision,
}
impl Layer {
    fn from_str(string: &str) -> Self {
        match string {
            "floor" => Self::Floor,
            "decor" => Self::Decor,
            "collision" => Self::Collision,
            _ => unreachable!(),
        }
    }
}
#[derive(Debug)]
struct Tile {
    textures: Vec<(u8, u8)>,
    layers: Vec<Layer>,
    collision: bool,
}
fn load_tilemap(tilemap: &str, tileset: &str) -> (Vec<Tile>, u32) {
    let tile_set_width = tileset
        .split_once("columns=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse::<u8>()
        .unwrap();
    dbg!(tile_set_width);
    #[derive(Debug)]
    struct Chunk {
        x: i32,
        y: i32,
        data: [u8; 256],
    }

    fn get_area(chunks: &Vec<Chunk>) -> (i32, i32, i32, i32) {
        let chunks: Vec<&Chunk> = chunks
            .iter()
            .filter(|f| !f.data.iter().all(|f| *f == 0))
            .collect();
        dbg!(&chunks);
        let posses: Vec<(i32, i32, i32, i32)> = chunks
            .iter()
            .map(|f| {
                let lowest_x = f.x
                    + f.data
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 % 16)
                        .min()
                        .unwrap() as i32;
                let highest_x = f.x
                    + f.data
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| {
                            dbg!(f, f.0 % 16);
                            f.0 % 16
                        })
                        .max()
                        .unwrap() as i32;
                dbg!(highest_x);
                let lowest_y = f.y
                    + f.data
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 / 16)
                        .min()
                        .unwrap() as i32;
                let highest_y = f.y
                    + f.data
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 / 16)
                        .max()
                        .unwrap() as i32;
                (lowest_x, lowest_y, highest_x, highest_y)
            })
            .collect();
        dbg!(&posses);
        let lowest_x = posses.iter().map(|f| f.0).min().unwrap_or(posses[0].0);
        let highest_x = posses.iter().map(|f| f.2).max().unwrap();
        let lowest_y = posses.iter().map(|f| f.1).min().unwrap_or(posses[0].1);
        let highest_y = posses.iter().map(|f| f.3).max().unwrap_or(posses[0].3);

        (lowest_x, lowest_y, highest_x, highest_y)
    }
    let mut layers: Vec<(Vec<Chunk>, &str)> = Vec::new();
    for layer in tilemap.split("<layer").skip(1) {
        let name = layer
            .split_once("name=\"")
            .unwrap()
            .1
            .split_once("\"")
            .unwrap()
            .0;
        dbg!(name);
        let mut chunks: Vec<Chunk> = Vec::new();
        for chunk in layer.split("<chunk").skip(1) {
            let x = chunk
                .split_once("x=\"")
                .unwrap()
                .1
                .split_once("\"")
                .unwrap()
                .0
                .parse::<i32>()
                .unwrap();
            let y = chunk
                .split_once("y=\"")
                .unwrap()
                .1
                .split_once("\"")
                .unwrap()
                .0
                .parse::<i32>()
                .unwrap();

            let chunk = chunk
                .split_once("\r\n")
                .unwrap()
                .1
                .split_once("\r\n</")
                .unwrap()
                .0;
            let mut data = [0; 256];

            for (index, id) in chunk.split(",").enumerate() {
                let id = if id.contains("\r\n") {
                    &id.replace("\r\n", "")
                } else {
                    id
                };

                data[index] = id.parse::<u8>().unwrap();
            }
            if data.iter().all(|f| *f == 0) {
                println!("chunk x: {},y: {} is empty ", x, y);
                continue;
            } else {
                println!("chunk is full of juice x: {}y:{}", x, y)
            }

            chunks.push(Chunk { x, y, data });
        }
        layers.push((chunks, name));
    }
    let layers_pos: Vec<(i32, i32, i32, i32)> = layers.iter().map(|f| get_area(&f.0)).collect();
    dbg!(&layers_pos);
    let area: (i32, i32, i32, i32) = (
        layers_pos.iter().map(|f| f.0).min().unwrap(),
        layers_pos.iter().map(|f| f.1).min().unwrap(),
        layers_pos.iter().map(|f| f.2).max().unwrap(),
        layers_pos.iter().map(|f| f.3).max().unwrap(),
    );
    dbg!(area);
    let mut tiles: Vec<Tile> = Vec::with_capacity(((area.2 - area.0) * (area.3 - area.1)) as usize);

    for y in area.1..area.3 + 1 {
        for x in area.0..area.2 + 1 {
            let mut tile = Tile {
                textures: vec![],
                collision: false,
                layers: Vec::new(),
            };
            for (chunks, name) in layers.iter() {
                if let Some(chunk) = chunks.iter().find(|f| {
                    f.x == ((x as f32 / 16.0).floor() * 16.0) as i32
                        && f.y == ((y as f32 / 16.0).floor() * 16.0) as i32
                }) {
                    let id = chunk.data[((y - chunk.y) * 16 + (x - chunk.x) % 16) as usize];

                    if id != 0 {
                        let id = id - 1;
                        if name.contains("collision") {
                            tile.collision = true;
                        }
                        tile.textures
                            .push((id % tile_set_width, id / tile_set_width));
                        dbg!(name);
                        tile.layers.push(Layer::from_str(&name));
                    }
                }
            }
            tiles.push(tile);
        }
    }
    (tiles, (area.2 + 1 - area.0) as u32)
}
struct Map {
    tiles: Vec<Tile>,
    width: u32,
}
impl Map {
    fn new() -> Self {
        let map = load_tilemap(
            include_str!("../assets/tilemap.tmx"),
            include_str!("../assets/spritesheet.tsx"),
        );
        Self {
            tiles: map.0,
            width: map.1,
        }
    }
    fn draw_map(&self) {
        for (index, tile) in self.tiles.iter().enumerate() {
            for text in &tile.textures {
                SPRITESHEET.draw_from(
                    vec2(
                        (index as u32 % self.width) as f32 * 16.0 * MAP_SCALE_FACTOR,
                        (index as u32 / self.width) as f32 * 16.0 * MAP_SCALE_FACTOR,
                    ),
                    *text,
                    MAP_SCALE_FACTOR,
                );
            }
        }
    }
}
static SPRITESHEET: LazyLock<Spritesheet> = std::sync::LazyLock::new(|| Spritesheet {
    texture: load_ase_texture(include_bytes!("../assets/spritesheet.ase"), None, None),
    widht: 16.0,
    height: 16.0,
});
static MOUSE_ANIMATION: LazyLock<Animation> = std::sync::LazyLock::new(|| {
    load_animation_from_tag(include_bytes!("../assets/mouse.ase"), "walk")
});
const MAP_SCALE_FACTOR: f32 = 3.0;
struct Spawner {
    clock: f32,
}
impl Spawner {
    fn new() -> Self {
        Self { clock: 0.0 }
    }
    fn spawn_wave(entities: &mut Vec<Mouse>, map: &Map) {
        println!("spawnin");
        let wave_size = 30;
        let mut dealt_with = Vec::with_capacity(30);
        while dealt_with.len() < wave_size {
            let rand = rand::gen_range(0, map.tiles.len());
            if dealt_with.contains(&rand) {
                continue;
            }
            let tile = &map.tiles[rand];
            if tile.layers.len() == 1 && tile.layers[0] == Layer::Floor {
                dealt_with.push(rand);
                let size = vec2(
                    MOUSE_ANIMATION.0[0].0.width(),
                    MOUSE_ANIMATION.0[0].0.height(),
                );
                let rainbow = if rand::gen_range(0, 50) == 0 {
                    true
                } else {
                    false
                };
                entities.push(Mouse {
                    speed: if rainbow { 5.0 } else { 2.0 },
                    is_rainbow: rainbow,
                    scare_clock: 0.0,
                    size,
                    pos: vec2(
                        (rand as u32 % map.width) as f32 * 16.0 * MAP_SCALE_FACTOR,
                        (rand as u32 / map.width) as f32 * 16.0 * MAP_SCALE_FACTOR,
                    ),
                    direction: Vec2::ZERO,
                    animation: &MOUSE_ANIMATION,
                });
            }
        }
    }
    fn update(&mut self, entities: &mut Vec<Mouse>, map: &Map) {
        self.clock -= get_frame_time();
        if self.clock <= 0.0 {
            self.clock = 10.0;
            Spawner::spawn_wave(entities, map)
        }
    }
}
#[derive(PartialEq)]
enum State {
    Menu,
    Game,
}
const DEFAULT_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;

uniform sampler2D Texture;

void main() {
    gl_FragColor = vec4(uv.xy,0.0,1.0);
}
";

const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";
static RAINBOW_SHADER: LazyLock<Material> = std::sync::LazyLock::new(|| {
    load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX_SHADER,
            fragment: DEFAULT_FRAGMENT_SHADER,
        },
        MaterialParams {
            ..Default::default()
        },
    )
    .unwrap()
});
struct Game<'a> {
    state: State,
    cat: Cat,
    mice: Vec<Mouse<'a>>,
    camera: Camera2D,
    map: Map,
    spawner: Spawner,
}
impl<'a> Game<'a> {
    fn new() -> Self {
        Self {
            state: State::Menu,
            spawner: Spawner::new(),
            map: Map::new(),
            cat: Cat::new(),
            mice: Vec::new(),
            camera: create_camera(SCREEN_SIZE),
        }
    }
    fn draw_mice(&self) {
        for mouse in self.mice.iter() {
            gl_use_material(&RAINBOW_SHADER);
            let mut time = (get_time() * 1000.0) % mouse.animation.1 as f64;
            for i in &mouse.animation.0 {
                if time <= i.1 as f64 {
                    draw_texture_ex(
                        &i.0,
                        mouse.pos.x,
                        mouse.pos.y,
                        WHITE,
                        DrawTextureParams {
                            rotation: mouse.direction.y.atan2(mouse.direction.x) + PI / 2.0,

                            ..Default::default()
                        },
                    );
                    break;
                } else {
                    time -= i.1 as f64;
                }
            }

            gl_use_default_material();
        }
    }
    fn draw_camera(&self) {
        set_default_camera();
        draw_texture_ex(
            &self.camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(
                    SCREEN_SIZE
                        * (screen_width() / SCREEN_SIZE.x).min(screen_height() / SCREEN_SIZE.y),
                ),
                ..Default::default()
            },
        );
        set_camera(&self.camera);
        clear_background(BLACK);
    }
    fn mouse_eatery(&mut self) {
        self.mice.retain(|f| {
            let collisions = [
                (0.0, 0.0),
                (f.size.x, 0.0),
                (0.0, f.size.y),
                (f.size.x, f.size.y),
            ];
            let mut collide = false;
            for p in collisions {
                if p.0 + f.pos.x >= self.cat.pos.x
                    && p.0 + f.pos.x <= self.cat.pos.x + self.cat.size.x
                    && f.pos.y + p.1 <= self.cat.pos.y + self.cat.size.y
                    && f.pos.y + p.1 > self.cat.pos.y
                {
                    collide = true;
                }
            }
            !collide
        })
    }
    fn mouse_behaviour(&mut self) {
        for mouse in self.mice.iter_mut() {
            let collisions = [
                (0.0, 0.0),
                (mouse.size.x, 0.0),
                (0.0, mouse.size.y),
                (mouse.size.x, mouse.size.y),
            ];

            for p in collisions {
                let map_pos =
                    (mouse.pos + mouse.direction + vec2(p.0, p.1)) / (16.0 * MAP_SCALE_FACTOR);
                if map_pos.y as u32 >= self.map.tiles.len() as u32 / self.map.width - 2 {
                    break;
                }

                let pottential_collider = &self.map.tiles
                    [map_pos.y as usize * self.map.width as usize + map_pos.x as usize];

                if pottential_collider.collision {
                    mouse.direction *= -1.0;
                    break;
                }
            }
            if mouse.scare_clock <= 0.0 {
                mouse.direction = (mouse.pos - self.cat.pos).normalize();
                mouse.scare_clock = 3.0;
            } else {
                mouse.scare_clock -= get_frame_time();
            }

            mouse.pos += mouse.direction * mouse.speed;
        }
    }
    async fn update(&mut self) {
        self.map.draw_map();
        self.draw_mice();
        self.mouse_eatery();
        self.mouse_behaviour();
        self.cat.update(&self.map);
        self.spawner.update(&mut self.mice, &self.map);
        self.camera.target = self.cat.pos;
        self.draw_camera();
    }
}
struct Button {
    rect: Rect,
    texture: Texture2D,
}
impl Button {
    fn is_clicked(&self, mouse_pos: (f32, f32)) -> bool {
        mouse_pos.0 >= self.rect.x
            && mouse_pos.0 <= self.rect.x + self.rect.w
            && mouse_pos.1 <= self.rect.h + self.rect.y
            && mouse_pos.1 >= self.rect.y
            && is_mouse_button_down(MouseButton::Left)
    }
}
struct Menu {
    size: (f32, f32),
    button: Button,
    background: Texture2D,
    cat: Vec<Animation>,
    timer: f32,
    animation_clock: f32,
    current_animation: Option<usize>,
    play: bool,
}

impl Menu {
    fn new() -> Self {
        let play = load_ase_texture(include_bytes!("../assets/play.ase"), None, None);
        let bsize = 0.2 * vec2(play.width(), play.height());
        let background = load_ase_texture(include_bytes!("../assets/background.ase"), None, None);
        dbg!(background.width());
        let size = (background.width(), background.height());
        Self {
            animation_clock: 0.0,
            current_animation: None,
            play: false,
            timer: 0.0,
            cat: vec![
                load_animation_from_tag(include_bytes!("../assets/main_menu_cat.ase"), "still"),
                load_animation_from_tag(include_bytes!("../assets/main_menu_cat.ase"), "lick"),
                load_animation_from_tag(include_bytes!("../assets/main_menu_cat.ase"), "scratch"),
            ],
            background,
            button: Button {
                texture: play,
                rect: Rect {
                    x: 20.0,
                    y: 80.0,
                    w: bsize.x,
                    h: bsize.y,
                },
            },
            size,
        }
    }
    async fn update(&mut self) {
        let sf = (screen_width() / self.size.0).min(screen_height() / self.size.1);

        draw_texture_ex(
            &self.background,
            0.0,
            0.00,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    self.background.width() * sf,
                    self.background.height() * sf,
                )),
                ..Default::default()
            },
        );
        draw_texture_ex(
            &self.button.texture,
            self.button.rect.x,
            self.button.rect.y,
            WHITE,
            DrawTextureParams {
                ..Default::default()
            },
        );
        if let Some(animation) = self.current_animation {
            let animation = &self.cat[animation];
            let clock = (self.animation_clock * 1000.0) as u32;
            if clock < animation.1 {
                let mut frame = clock;
                for i in animation.0.iter() {
                    if frame > i.1 {
                        frame -= i.1
                    } else {
                        draw_texture_ex(
                            &i.0,
                            140.0 * sf,
                            70.0 * sf,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(i.0.width() * sf, i.0.height() * sf)),
                                ..Default::default()
                            },
                        );
                        break;
                    }
                }
                self.animation_clock += get_frame_time();
            } else {
                self.current_animation = None;
                self.animation_clock = 0.0;
            }
        } else {
            let text = &self.cat[0].0[0].0;
            draw_texture_ex(
                text,
                140.0 * sf,
                70.0 * sf,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(text.width() * sf, text.height() * sf)),

                    ..Default::default()
                },
            );
        }
        if self.timer <= 0.0 {
            self.timer = 7.0;
            self.current_animation = Some(rand::gen_range(1, self.cat.len()));
        } else {
            self.timer -= get_frame_time();
        }
        let mouse_pos = mouse_position();
        let mouse_pos = (mouse_pos.0 / sf, mouse_pos.1 / sf);
        if self.button.is_clicked(mouse_pos) {
            self.play = true;
        }
    }
}
struct GameManager<'a> {
    menu: Menu,
    game: Game<'a>,
    state: State,
}
impl<'a> GameManager<'a> {
    fn new() -> Self {
        Self {
            state: State::Menu,
            game: Game::new(),
            menu: Menu::new(),
        }
    }
    async fn update(&mut self) {
        match self.state {
            State::Game => self.game.update().await,
            State::Menu => {
                if self.menu.play {
                    self.state = State::Game
                } else {
                    self.menu.update().await
                }
            }
        }
    }
}
#[macroquad::main("catscapade")]
async fn main() {
    let mut game = GameManager::new();
    loop {
        game.update().await;
        next_frame().await;
    }
}
