use std::sync::LazyLock;

use asefile::AsepriteFile;
use image::*;
use macroquad::prelude::*;
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
        target: vec2(dimensions.x / 2.0, dimensions.y / 2.0),
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
    fn draw_from(&self, world_pos: Vec2, texture_coord: (u8, u8)) {
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
                ..Default::default()
            },
        )
    }
}
type Animation = (Vec<(Texture2D, u32)>, u32);

struct PlayerAnimations {
    walk: Animation,
}
struct EntityAnimations {
    walk: Animation,
}
struct Player {
    pos: Vec2,
    direction: Vec2,
    animations: PlayerAnimations,
}
impl Player {
    fn new() -> Self {
        Self {
            pos: Vec2::ZERO,
            direction: Vec2::ZERO,
            animations: PlayerAnimations {
                walk: load_animation_from_tag(include_bytes!("../assets/cat.ase"), "walk"),
            },
        }
    }
    fn update(&mut self) {
        let mut animation = &self.animations.walk;
        let mut flipped = (false, false);
        if is_key_down(KeyCode::A) {
            self.direction.x += -1.0;
            animation = &self.animations.walk;
            flipped.0 = true;
        }
        if is_key_down(KeyCode::D) {
            self.direction.x += 1.0;
            animation = &self.animations.walk;
        }
        if is_key_down(KeyCode::S) {
            self.direction.y += 1.0;
            animation = &self.animations.walk;
            flipped.1 = true;
        }
        if is_key_down(KeyCode::W) {
            self.direction.y += -1.0;
            animation = &self.animations.walk;
        }
        self.direction = self.direction.lerp(Vec2::ZERO, 0.5);
        self.pos += self.direction;
        let mut time = (get_time() * 1000.0) % animation.1 as f64;
        for i in &animation.0 {
            if time <= i.1 as f64 {
                draw_texture_ex(
                    &i.0,
                    self.pos.x,
                    self.pos.y,
                    WHITE,
                    DrawTextureParams {
                        flip_x: flipped.0,
                        flip_y: flipped.1,
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
struct Entity {
    pos: Vec2,
}
const SCREEN_SIZE: Vec2 = Vec2 { x: 160.0, y: 160.0 };
#[derive(Debug)]
struct Tile {
    textures: Vec<(u8, u8)>,
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
    let mut layers: Vec<Vec<Chunk>> = Vec::new();
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
        layers.push(chunks);
    }
    let layers_pos: Vec<(i32, i32, i32, i32)> = layers.iter().map(|f| get_area(f)).collect();
    dbg!(&layers_pos);
    let area: (i32, i32, i32, i32) = (
        layers_pos.iter().map(|f| f.0).min().unwrap(),
        layers_pos.iter().map(|f| f.1).min().unwrap(),
        layers_pos.iter().map(|f| f.2).max().unwrap(),
        layers_pos.iter().map(|f| f.3).max().unwrap(),
    );
    dbg!(area);
    let mut tiles: Vec<Tile> = Vec::with_capacity(((area.2 - area.0) * (area.3 - area.1)) as usize);

    for y in area.1..area.3 {
        for x in area.0..area.2 {
            let mut tile = Tile {
                textures: vec![(2, 0)],
                collision: true,
            };
            for (index, layer) in layers.iter().enumerate() {
                let layer_pos = layers_pos[index];
                if x >= layer_pos.0 && layer_pos.2 > x && layer_pos.1 <= y && y < layer_pos.3 {
                    let chunk = layer
                        .iter()
                        .find(|f| {
                            f.x == ((x as f32 / 16.0).floor() * 16.0) as i32
                                && f.y == ((y as f32 / 16.0).floor() * 16.0) as i32
                        })
                        .unwrap();

                    // dbg!(chunk.x, chunk.y, x, y);
                    let id = chunk.data[((y - chunk.y) * 16 + (x - chunk.x) % 16) as usize];
                    // dbg!(id);
                    if id != 0 {
                        let id = id - 1;
                        tile.textures
                            .push((id % tile_set_width, id / tile_set_width));
                    }
                } else {
                    println!(
                        "it seems there isnt a chunk at x:{} y:{} but i think there should be",
                        x, y
                    );
                }
            }
            tiles.push(tile);
        }
    }
    (tiles, (area.2 - area.0) as u32)
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
                        (index as u32 % self.width) as f32 * 16.0,
                        (index as u32 / self.width) as f32 * 16.0,
                    ),
                    *text,
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
struct Game {
    cat: Player,
    entities: Vec<Entity>,
    camera: Camera2D,
    map: Map,
}
impl Game {
    fn new() -> Self {
        Self {
            map: Map::new(),
            cat: Player::new(),
            entities: Vec::new(),
            camera: create_camera(SCREEN_SIZE),
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
    async fn update(&mut self) {
        #[cfg(debug_assertions)]
        {
            if is_key_down(KeyCode::Left) {
                self.camera.target.x -= 4.0;
            }
            if is_key_down(KeyCode::Right) {
                self.camera.target.x += 4.0;
            }
            if is_key_down(KeyCode::Down) {
                self.camera.target.y += 4.0;
            }
            if is_key_down(KeyCode::Up) {
                self.camera.target.y -= 4.0;
            }
        }
        self.cat.update();
        self.map.draw_map();
        self.draw_camera();
    }
}
#[macroquad::main("catscapade")]
async fn main() {
    let mut game = Game::new();
    loop {
        game.update().await;
        next_frame().await;
    }
}
