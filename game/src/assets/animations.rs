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

#[derive(Default, Copy, Clone)]
pub struct WarriorAnimation {
    pub idle: AnimationBase,
    pub walk: AnimationBase,
    pub strike_h1: AnimationBase,
    pub strike_h2: AnimationBase,
    pub strike_b1: AnimationBase,
    pub strike_b2: AnimationBase,
    pub strike_t1: AnimationBase,
    pub strike_t2: AnimationBase,
}

impl WarriorAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "idle" => Some(&mut self.idle),
            "walk" => Some(&mut self.walk),
            "strike-horz-1" => Some(&mut self.strike_h1),
            "strike-horz-2" => Some(&mut self.strike_h2),
            "strike-bottom-1" => Some(&mut self.strike_b1),
            "strike-bottom-2" => Some(&mut self.strike_b2),
            "strike-top-1" => Some(&mut self.strike_t1),
            "strike-top-2" => Some(&mut self.strike_t2),
            _ => None,
        }?;

        *target = animation;

        Some(())
    }
}

#[derive(Default, Copy, Clone)]
pub struct ArcherAnimation {
    pub idle: AnimationBase,
    pub walk: AnimationBase,
    pub fire_top: AnimationBase,
    pub fire_top_h: AnimationBase,
    pub fire_h: AnimationBase,
    pub fire_bottom_h: AnimationBase,
    pub fire_bottom: AnimationBase,
}

impl ArcherAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "idle" => Some(&mut self.idle),
            "walk" => Some(&mut self.walk),
            "fire-top" => Some(&mut self.fire_top),
            "fire-top-horz" => Some(&mut self.fire_top_h),
            "fire-horz" => Some(&mut self.fire_h),
            "fire-bottom-horz" => Some(&mut self.fire_bottom_h),
            "fire-bottom" => Some(&mut self.fire_bottom),
            _ => None,
        }?;

        *target = animation;

        Some(())
    }
}

#[derive(Default, Copy, Clone)]
pub struct TorchGoblinAnimation {
    pub idle: AnimationBase,
    pub walk: AnimationBase,
    pub strike_h: AnimationBase,
    pub strike_b: AnimationBase,
    pub strike_t: AnimationBase,
}


impl TorchGoblinAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "idle" => Some(&mut self.idle),
            "walk" => Some(&mut self.walk),
            "strike-horz" => Some(&mut self.strike_h),
            "strike-bottom" => Some(&mut self.strike_b),
            "strike-top" => Some(&mut self.strike_t),
            _ => None,
        }?;

        *target = animation;

        Some(())
    }
}


#[derive(Default, Copy, Clone)]
pub struct TntGoblinAnimation {
    pub idle: AnimationBase,
    pub walk: AnimationBase,
    pub throw: AnimationBase,
}

impl TntGoblinAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "idle" => Some(&mut self.idle),
            "walk" => Some(&mut self.walk),
            "throw" => Some(&mut self.throw),
            _ => None,
        }?;

        *target = animation;

        Some(())
    }
}

#[derive(Default, Copy, Clone)]
pub struct SheepAnimation {
    pub idle: AnimationBase,
    pub walk: AnimationBase,
}

impl SheepAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "idle" => Some(&mut self.idle),
            "walk" => Some(&mut self.walk),
            _ => None,
        }?;

        *target = animation;

        Some(())
    }
}


#[derive(Default, Clone, Copy)]
pub struct AnimationsBundle {
    pub pawn: PawnAnimation,
    pub warrior: WarriorAnimation,
    pub archer: ArcherAnimation,
    pub torch_goblin: TorchGoblinAnimation,
    pub tnt_goblin: TntGoblinAnimation,
    pub sheep: SheepAnimation,
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
                "warrior_sprites" => self.warrior.set_animation_by_name(animation_name, animation),
                "archer_sprites" => self.archer.set_animation_by_name(animation_name, animation),
                "torch_goblins_sprites" => self.torch_goblin.set_animation_by_name(animation_name, animation),
                "tnt_goblin_sprites" => self.tnt_goblin.set_animation_by_name(animation_name, animation),
                "sheep_sprites" => self.sheep.set_animation_by_name(animation_name, animation),
                _ => { 
                    warn!("Unknown animation group name: {:?}", name);
                    continue;
                }
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
