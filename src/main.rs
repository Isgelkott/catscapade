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
    fn draw_from(&self, world_pos: Vec2, texture_coord: (f32, f32)) {
        draw_texture_ex(
            &self.texture,
            world_pos.x,
            world_pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect {
                    x: texture_coord.0 as f32 * self.widht,
                    y: texture_coord.1 * self.height,
                    w: self.widht,
                    h: self.height,
                }),
                ..Default::default()
            },
        )
    }
}
struct PlayerAnimations {
    walk_up: (Vec<(Texture2D, u32)>, u32),
    walk_down: (Vec<(Texture2D, u32)>, u32),
    walk_side: (Vec<(Texture2D, u32)>, u32),
    idle: (Vec<(Texture2D, u32)>, u32),
}
struct EntityAnimations {
    walk: (Vec<(Texture2D, u32)>, u32),
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
                walk_up: load_animation_from_tag(include_bytes!("../assets/cat.ase"), "walk_up"),
                walk_down: load_animation_from_tag(include_bytes!("../assets/cat.ase"), "walk_up"),
                walk_side: load_animation_from_tag(include_bytes!("../assets/cat.ase"), "walk_up"),
                idle: load_animation_from_tag(include_bytes!("../assets/cat.ase"), "idle"),
            },
        }
    }
    fn update(&mut self) {}
}
struct Entity {
    pos: Vec2,
}
struct Game {
    spritesheet: Spritesheet,
    cat: Player,
    entities: Vec<Entity>,
}
impl Game {
    fn new() -> Self {
        Self {
            spritesheet: Spritesheet {
                texture: load_ase_texture(include_bytes!("../assets/spritesheet.ase"), None, None),
                widht: 16.0,
                height: 16.0,
            },
        }
    }
}
#[macroquad::main("catscapade")]
async fn main() {}
