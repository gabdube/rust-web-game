use serde_json::Value as JsonValue;
use crate::error::Error;

#[derive(Copy, Clone, Debug, Default)]
pub struct AnimationBase {
    pub padding: f32,
    pub x: f32,
    pub y: f32,
    pub sprite_width: f32,
    pub sprite_height: f32,
    pub last_frame: u8,
}

#[derive(Default, Copy, Clone, PartialEq)]
pub enum PawnAnimationType {
    #[default]
    Idle,
    Walk,
    Hammer,
    Axe,
    IdleHold,
    IdleWalk
}

#[derive(Default, Copy, Clone)]
pub struct PawnAnimation {
    pub idle: AnimationBase,
    pub walk: AnimationBase,
    pub hammer: AnimationBase,
    pub axe: AnimationBase,
    pub idle_hold: AnimationBase,
    pub idle_walk: AnimationBase,
}

impl PawnAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "idle" => Some(&mut self.idle),
            "walk" => Some(&mut self.walk),
            "hammer" => Some(&mut self.hammer),
            "axe" => Some(&mut self.axe),
            "idle_hold" => Some(&mut self.idle_hold),
            "idle_walk" => Some(&mut self.idle_walk),
            _ => None,
        }?;

        *target = animation;
        Some(())
    }
}


#[derive(Default, Clone, Copy)]
pub struct AnimationsBundle {
    pub pawn: PawnAnimation,
}

impl AnimationsBundle {

    pub fn load_animation(&mut self, name: &str, json: JsonValue) -> Result<(), Error>{
        let animations = json.get("animations")
            .and_then(|value| value.as_array() )
            .ok_or_else(|| assets_err!("Missing json key \"animations\"") )?;

        for animation in animations {
            let animation_name = animation.get("name")
                .and_then(|value| value.as_str() )
                .ok_or_else(|| assets_err!("Missing json key \"animations.name\"") )?;

            let sprite_count: u32 = parse_u32(&animation["count"]);
            let padding: f32 = parse_f32(&animation["padding"]);
            let x: f32 = parse_f32(&animation["x"]);
            let y: f32 = parse_f32(&animation["y"]);
            let sprite_width: f32 = parse_f32(&animation["width"]);
            let sprite_height: f32 = parse_f32(&animation["height"]);

            let animation = AnimationBase {
                padding,
                x, y,
                sprite_width, sprite_height,
                last_frame: (sprite_count - 1) as u8,
            };

            let result = match name {
                "pawn_sprites" => self.pawn.set_animation_by_name(animation_name, animation),
                _ => { return Err(assets_err!("Unknown animation group name: {:?}", name)); }
            };

            if result.is_none() {
                return Err(assets_err!("Unknown animation {:?} for animation group {:?}", animation_name, name));
            }
        }

        Ok(())
    }

}

#[inline]
fn parse_u32(item: &JsonValue) -> u32 {
    item.as_u64().map(|v| v as u32).unwrap_or(0)
}

#[inline]
fn parse_f32(item: &JsonValue) -> f32 {
    item.as_f64().map(|v| v as f32).unwrap_or(0.0)
}
