use asefile::AsepriteFile;
use image::*;
use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};
use std::{collections::HashMap, f32::consts::PI, sync::LazyLock, vec};
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
const CAT_SPEED: f32 = 200.0;
struct Cat {
    pos: Vec2,
    size: Vec2,
    direction: Vec2,
    animations: PlayerAnimations,
    last_rotation: f32,
}
impl Cat {
    fn new() -> Self {
        let animations = PlayerAnimations {
            walk: load_animation_from_tag(include_bytes!("../assets/cat.ase"), "walk"),
        };

        Self {
            last_rotation: 0.0,
            pos: vec2(750.0, 250.0),
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
        let rotation = if direction == Vec2::ZERO {
            self.last_rotation
        } else {
            0.5 * PI + direction.y.atan2(direction.x)
        };
        self.last_rotation = rotation;
        self.direction += direction.normalize_or_zero();
        let shrunk_collision = 4.0;
        let collision_points = [
            (shrunk_collision, shrunk_collision),
            (self.size.x - shrunk_collision, shrunk_collision),
            (shrunk_collision, self.size.y - shrunk_collision),
            (
                self.size.x - shrunk_collision,
                self.size.y - shrunk_collision,
            ),
        ];

        for p in collision_points.iter() {
            let map_pos = (self.pos + self.direction + vec2(p.0, p.1)) / (16.0 * MAP_SCALE_FACTOR);
            let pottential_collider =
                &map.tiles[map_pos.y as usize * map.width as usize + map_pos.x as usize];

            if pottential_collider.collision {
                let x0 = map_pos.x.floor() * 16.0 * MAP_SCALE_FACTOR - p.0;
                let x1 = (map_pos.x.floor() + 1.0) * MAP_SCALE_FACTOR * 16.0 - p.0;
                let y0 = map_pos.y.floor() * 16.0 * MAP_SCALE_FACTOR - p.1;
                let y1 = map_pos.y.ceil() * MAP_SCALE_FACTOR * 16.0 - p.1;
                if self.pos.y + self.direction.y != y0 {
                    self.pos.x = self.pos.x.clamp(x0, x1);
                    self.pos.y = self.pos.y.clamp(y0, y1);
                    if self.pos.x == x0 || self.pos.x == x1 {
                        self.direction.x = 0.0;
                    } else if self.pos.y == y0 || self.pos.y == y1 {
                        self.direction.y = 0.0;
                    }
                }

                // break;
            }
        }
        self.pos += self.direction.normalize_or_zero() * CAT_SPEED * get_frame_time();

        self.direction *= 0.8;
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
    scare_timer: f32,
    random_direction_cooldown: f32,
    is_rainbow: bool,
    pos: Vec2,
    speed: f32,
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
    fn get_area(chunks: &HashMap<(i32, i32), [u8; 256]>) -> (i32, i32, i32, i32) {
        let posses: Vec<(i32, i32, i32, i32)> = chunks
            .iter()
            .map(|f| {
                let lowest_x = f.0.0
                    + f.1
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 % 16)
                        .min()
                        .unwrap() as i32;
                let highest_x = f.0.0
                    + f.1
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 % 16)
                        .max()
                        .unwrap() as i32;
                let lowest_y = f.0.1
                    + f.1
                        .iter()
                        .enumerate()
                        .filter(|f| *f.1 != 0)
                        .map(|f| f.0 / 16)
                        .min()
                        .unwrap() as i32;
                let highest_y = f.0.1
                    + f.1
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
    let mut layers: Vec<(HashMap<(i32, i32), [u8; 256]>, &str)> = Vec::new();
    for layer in tilemap.split("<layer").skip(1) {
        let name = layer
            .split_once("name=\"")
            .unwrap()
            .1
            .split_once("\"")
            .unwrap()
            .0;
        dbg!(name);
        let mut chunks: HashMap<(i32, i32), [u8; 256]> = HashMap::new();
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

            chunks.insert((x, y), data);
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
                if let Some(chunk) = chunks.get(&(
                    ((x as f32 / 16.0).floor() * 16.0) as i32,
                    ((y as f32 / 16.0).floor() * 16.0) as i32,
                )) {
                    let id = chunk[(y % 16 * 16 + x % 16) as usize];

                    if id != 0 {
                        let id = id - 1;
                        if name.contains("collision") {
                            tile.collision = true;
                        }
                        tile.textures
                            .push((id % tile_set_width, id / tile_set_width));
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
                let rainbow = if rand::gen_range(0, 30) == 0 {
                    true
                } else {
                    false
                };
                entities.push(Mouse {
                    speed: if rainbow { 250.0 } else { 150.0 },
                    scare_timer: 0.0,
                    random_direction_cooldown: 0.0,

                    is_rainbow: rainbow,
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
            Self::spawn_wave(entities, map)
        }
    }
}
#[derive(PartialEq)]
enum State {
    Menu,
    Game,
}
const RAIINBOW_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;

uniform sampler2D Texture;
uniform lowp float time;

void main() {
    
    if (texture2D(Texture, uv).b > 0.04){

    gl_FragColor = vec4((sin(uv.x + time)+1.0)/2.0,(sin(uv.y + time)+1.0)/2.0, 0.65,1.0);
    }else{

    gl_FragColor = texture2D(Texture, uv);
    }
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
    let pipeline = PipelineParams {
        alpha_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX_SHADER,
            fragment: RAIINBOW_FRAGMENT_SHADER,
        },
        MaterialParams {
            pipeline_params: pipeline,
            uniforms: vec![UniformDesc::new("time", UniformType::Float1)],
            ..Default::default()
        },
    )
    .unwrap()
});
static FONT: LazyLock<Font> =
    LazyLock::new(|| load_ttf_font_from_bytes(include_bytes!("../assets/GOUDYSTO.TTF")).unwrap());

struct Debug {
    mouse_cam: bool,
    mouse: usize,
}
static mut DEBUG: Debug = Debug {
    mouse_cam: true,
    mouse: 0,
};
struct Game<'a> {
    cat: Cat,
    mice: Vec<Mouse<'a>>,
    camera: Camera2D,
    map: Map,
    spawner: Spawner,
    timer: f32,
    fade_out_clock: f32,
    done: bool,
    go_back_button: Button,
    go_to_menu: bool,
    kills: u32,
    clock: Texture2D,
    mouse_icon: Texture2D,
}
impl<'a> Game<'a> {
    fn new() -> Self {
        let button = load_ase_texture(include_bytes!("../assets/back.ase"), None, None);
        Self {
            mouse_icon: load_ase_texture(include_bytes!("../assets/mouse_icon.ase"), None, None),
            clock: load_ase_texture(include_bytes!("../assets/clock.aseprite"), None, None),
            go_back_button: Button {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: button.width(),
                    h: button.height(),
                },
                texture: button,
            },
            go_to_menu: false,
            kills: 0,
            done: false,
            fade_out_clock: 0.0,
            timer: 30.0,
            spawner: Spawner::new(),
            map: Map::new(),
            cat: Cat::new(),
            mice: Vec::new(),
            camera: create_camera(SCREEN_SIZE),
        }
    }
    fn draw_mice(&self) {
        for mouse in self.mice.iter() {
            if mouse.is_rainbow {
                gl_use_material(&RAINBOW_SHADER);
            }
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
            if mouse.is_rainbow {
                gl_use_default_material();
            }
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
                    self.kills += if f.is_rainbow { 3 } else { 1 };
                }
            }

            !collide
        })
    }
    fn mouse_behaviour(&mut self) {
        for mouse in self.mice.iter_mut() {
            mouse.scare_timer = (mouse.scare_timer - get_frame_time()).max(0.0);
            if (((mouse.pos.x - self.cat.pos.x).powi(2) + (mouse.pos.y - self.cat.pos.y).powi(2))
                .sqrt())
            .abs()
                < 100.0
                && mouse.scare_timer == 0.0
            {
                mouse.scare_timer = if mouse.is_rainbow { 0.5 } else { 0.3 };
                mouse.direction = (mouse.pos - self.cat.pos).normalize_or_zero();
            } else if mouse.random_direction_cooldown < 0.0 {
                mouse.direction = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0))
                    .normalize_or_zero();
                mouse.random_direction_cooldown = rand::gen_range(1.0, 5.0);
            } else {
                mouse.random_direction_cooldown -= get_frame_time();
            }
            let collisions = [
                (0.0, 0.0),
                (mouse.size.x, 0.0),
                (0.0, mouse.size.y),
                (mouse.size.x, mouse.size.y),
            ];
            for p in collisions.iter() {
                let map_pos = (mouse.pos + mouse.direction * mouse.speed + vec2(p.0, p.1))
                    / (16.0 * MAP_SCALE_FACTOR);
                if self.map.width * map_pos.y as u32 + map_pos.x as u32
                    >= self.map.tiles.len() as u32
                {
                    break;
                }
                let map_pos =
                    (mouse.pos + mouse.direction + vec2(p.0, p.1)) / (16.0 * MAP_SCALE_FACTOR);
                let pottential_collider = &self.map.tiles
                    [map_pos.y as usize * self.map.width as usize + map_pos.x as usize];

                if pottential_collider.collision {
                    let x0 = map_pos.x.floor() * 16.0 * MAP_SCALE_FACTOR - p.0;
                    let x1 = (map_pos.x.floor() + 1.0) * MAP_SCALE_FACTOR * 16.0 - p.0;
                    let y0 = map_pos.y.floor() * 16.0 * MAP_SCALE_FACTOR - p.1;
                    let y1 = map_pos.y.ceil() * MAP_SCALE_FACTOR * 16.0 - p.1;
                    if mouse.pos.y + mouse.direction.y != y0 {
                        mouse.pos.x = mouse.pos.x.clamp(x0, x1);
                        mouse.pos.y = mouse.pos.y.clamp(y0, y1);
                        if mouse.pos.x == x0 || mouse.pos.x == x1 {
                            mouse.direction.x = 0.0;
                        } else if mouse.pos.y == y0 || mouse.pos.y == y1 {
                            mouse.direction.y = 0.0;
                        }
                    }
                }
            }

            mouse.pos += mouse.direction * mouse.speed * get_frame_time();
        }
    }
    fn fade_out_menu(&mut self) {
        set_default_camera();
        clear_background(BLACK);
        self.go_back_button.rect.x = (screen_width() - self.go_back_button.rect.w) / 2.0;
        self.go_back_button.rect.y = (screen_height() - self.go_back_button.rect.h + 300.0) / 2.0;
        draw_texture(
            &self.go_back_button.texture,
            self.go_back_button.rect.x,
            self.go_back_button.rect.y,
            WHITE,
        );
        let font_size = 30;
        draw_text_ex(
            "Good work soldier!",
            (screen_width() - 590.0) / 2.0,
            (screen_height() - 60.0) / 2.0,
            TextParams {
                font: Some(&FONT),
                font_size,
                ..Default::default()
            },
        );
        draw_text_ex(
            "You caught ",
            (screen_width() - 550.0) / 2.0,
            screen_height() / 2.0,
            TextParams {
                font: Some(&FONT),
                font_size,
                ..Default::default()
            },
        );
        draw_text_ex(
            &format!("{} mice!", self.kills),
            (screen_width() + 200.0) / 2.0,
            screen_height() / 2.0,
            TextParams {
                font: Some(&FONT),
                font_size,
                color: Color::from_hex(0xfbf236),
                ..Default::default()
            },
        );

        if self.go_back_button.is_clicked(mouse_position()) {
            self.go_to_menu = true;
        }
    }
    fn draw_hud(&self) {
        set_default_camera();

        draw_text(
            &((self.timer as i32).max(0)).to_string(),
            screen_width() - 120.00,
            40.0,
            60.0,
            WHITE,
        );
        draw_texture_ex(
            &self.clock,
            screen_width() - 60.,
            -7.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.clock.width(), self.clock.height()) * 4.0),
                ..Default::default()
            },
        );
        draw_text(&self.kills.to_string(), 80.0, 55.0, 60.0, WHITE);
        draw_texture_ex(
            &self.mouse_icon,
            5.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.mouse_icon.width(), self.mouse_icon.height()) * 4.0),
                ..Default::default()
            },
        );
        set_camera(&self.camera);
    }
    async fn update(&mut self) {
        if self.done {
            self.fade_out_menu();
        } else {
            self.map.draw_map();
            RAINBOW_SHADER.set_uniform("time", get_time() as f32 * 8.0);
            self.draw_mice();
            self.mouse_eatery();
            if is_key_pressed(KeyCode::G) {
                self.mice.push(Mouse {
                    speed: 150.0,
                    scare_timer: 0.0,
                    random_direction_cooldown: 0.0,
                    is_rainbow: false,
                    pos: vec2(200.0, 200.0),
                    size: vec2(
                        MOUSE_ANIMATION.0[0].0.width(),
                        MOUSE_ANIMATION.0[0].0.height(),
                    ),
                    direction: Vec2::ZERO,
                    animation: &MOUSE_ANIMATION,
                });
            }
            self.mouse_behaviour();
            self.cat.update(&self.map);
            // self.spawner.update(&mut self.mice, &self.map);
            unsafe {
                if DEBUG.mouse_cam && is_mouse_button_pressed(MouseButton::Left) {
                    DEBUG.mouse += 1;
                }
                if DEBUG.mouse_cam && self.mice.len() > 0 {
                    self.camera.target = self.mice[DEBUG.mouse].pos;
                } else {
                    self.camera.target = self.cat.pos;
                }
            }
            self.draw_camera();
            self.draw_hud();
            if self.timer <= 0.0 {
                self.fade_out_clock += get_frame_time();
                let fade_out = 2.0;
                if self.fade_out_clock < fade_out {
                    set_default_camera();
                    draw_rectangle(
                        0.0,
                        0.0,
                        screen_width(),
                        screen_height(),
                        BLACK.with_alpha(self.fade_out_clock / fade_out),
                    );
                    set_camera(&self.camera);
                } else {
                    self.done = true;
                }
            } else {
                self.timer -= get_frame_time()
            }
        }
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
    animation_timer: f32,
    animation_clock: f32,
    current_animation: Option<usize>,
    play: bool,
    high_score: u32,
}

impl Menu {
    fn new() -> Self {
        let high_score = quad_storage::LocalStorage::default()
            .get("high_score")
            .unwrap_or("0".to_string())
            .parse::<u32>()
            .unwrap_or_default();
        let play = load_ase_texture(include_bytes!("../assets/play.ase"), None, None);
        let bsize = 0.2 * vec2(play.width(), play.height());
        let background = load_ase_texture(include_bytes!("../assets/background.ase"), None, None);
        dbg!(background.width());
        let size = (background.width(), background.height());
        Self {
            high_score,
            animation_clock: 0.0,
            current_animation: None,
            play: false,
            animation_timer: 0.0,
            cat: vec![
                load_animation_from_tag(include_bytes!("../assets/main_menu_cat.ase"), "still"),
                load_animation_from_tag(include_bytes!("../assets/main_menu_cat.ase"), "blink"),
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
            self.button.rect.x * sf,
            self.button.rect.y * sf,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.button.rect.w, self.button.rect.h) * sf),
                ..Default::default()
            },
        );
        let cat_pos = vec2(100.0, 82.0);
        let draw_still = || {
            let texture = &self.cat[0].0[0].0;
            draw_texture_ex(
                texture,
                cat_pos.x * sf,
                cat_pos.y * sf,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(
                        texture.width() * sf * 1.5,
                        texture.height() * sf * 1.5,
                    )),

                    ..Default::default()
                },
            );
        };
        draw_text(
            &format!("High score: {}", self.high_score.to_string()),
            10.0 * sf,
            (self.background.height() - 10.0) * sf,
            20.0 * sf,
            WHITE,
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
                            cat_pos.x * sf,
                            cat_pos.y * sf,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(
                                    i.0.width() * sf * 1.5,
                                    i.0.height() * sf * 1.5,
                                )),
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
                (draw_still)()
            }
        } else {
            (draw_still)()
        }
        if self.animation_timer <= 0.0 {
            self.animation_timer = 7.0;
            self.current_animation = Some(rand::gen_range(1, self.cat.len()));
        } else {
            self.animation_timer -= get_frame_time();
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
    game: Option<Game<'a>>,
    state: State,
}
impl<'a> GameManager<'a> {
    fn new() -> Self {
        Self {
            state: State::Menu,
            game: None,
            menu: Menu::new(),
        }
    }
    async fn update(&mut self) {
        match self.state {
            State::Game => {
                let game = self.game.as_mut().unwrap();
                if game.go_to_menu {
                    self.state = State::Menu;
                    self.menu = Menu::new();
                    if game.kills > self.menu.high_score {
                        self.menu.high_score = game.kills;
                        let mut storage = quad_storage::LocalStorage::default();
                        storage.set("high_score", &game.kills.to_string());
                    }
                    self.game = None;
                } else {
                    game.update().await;
                }
            }
            State::Menu => {
                if self.menu.play {
                    self.state = State::Game;
                    self.game = Some(Game::new())
                } else {
                    self.menu.update().await
                }
            }
        }
    }
}
fn conf() -> Conf {
    Conf {
        window_title: String::from("catscapade"),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}
#[macroquad::main(conf)]
async fn main() {
    let mut game = GameManager::new();
    rand::srand(get_time() as u64);
    loop {
        game.update().await;
        next_frame().await;
    }
}
