use crate::error::Error;
use crate::shared::{AABB, split_csv};

#[derive(Copy, Clone, Debug, Default)]
pub struct AnimationBase {
    pub x: f32,
    pub y: f32,
    pub sprite_width: f32,
    pub sprite_height: f32,
    pub last_frame: u8,
}

impl AnimationBase {
    pub fn from_aabb(aabb: AABB) -> Self {
        AnimationBase {
            x: aabb.left,
            y: aabb.top,
            sprite_width: aabb.width(),
            sprite_height: aabb.height(),
            last_frame: 0,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct PawnAnimation {
    pub idle: AnimationBase,
    pub walk: AnimationBase,
    pub hammer: AnimationBase,
    pub axe: AnimationBase,
    pub idle_hold: AnimationBase,
    pub walk_hold: AnimationBase,
}

impl PawnAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "pawn_idle" => Some(&mut self.idle),
            "pawn_walk" => Some(&mut self.walk),
            "pawn_hammer" => Some(&mut self.hammer),
            "pawn_axe" => Some(&mut self.axe),
            "pawn_idle_hold" => Some(&mut self.idle_hold),
            "pawn_walk_hold" => Some(&mut self.walk_hold),
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
}

impl WarriorAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "warrior_idle" => Some(&mut self.idle),
            "warrior_walk" => Some(&mut self.walk),
            "warrior_strike_horz1" => Some(&mut self.strike_h1),
            "warrior_strike_horz2" => Some(&mut self.strike_h2),
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
            "archer_idle" => Some(&mut self.idle),
            "archer_walk" => Some(&mut self.walk),
            "archer_shoot_top" => Some(&mut self.fire_top),
            "archer_shoot_top_horz" => Some(&mut self.fire_top_h),
            "archer_shoot_horz" => Some(&mut self.fire_h),
            "archer_shoot_bottom_horz" => Some(&mut self.fire_bottom_h),
            "archer_shoot_bottom" => Some(&mut self.fire_bottom),
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
}

impl TorchGoblinAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "gobintorch_idle" => Some(&mut self.idle),
            "gobintorch_walk" => Some(&mut self.walk),
            "gobintorch_strike_horz" => Some(&mut self.strike_h),
            _ => None,
        }?;

        *target = animation;

        Some(())
    }
}

#[derive(Default, Copy, Clone)]
pub struct DynamiteGoblinAnimation {
    pub idle: AnimationBase,
    pub walk: AnimationBase,
    pub throw: AnimationBase,
}

impl DynamiteGoblinAnimation {
    fn set_animation_by_name(&mut self, name: &str, animation: AnimationBase) -> Option<()> {
        let target = match name {
            "gobindynamite_idle" => Some(&mut self.idle),
            "gobindynamite_walk" => Some(&mut self.walk),
            "gobindynamite_throw" => Some(&mut self.throw),
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
            "sheep_idle" => Some(&mut self.idle),
            "sheep_walk" => Some(&mut self.walk),
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
    pub dynamite_goblin: DynamiteGoblinAnimation,
    pub sheep: SheepAnimation,
}

impl AnimationsBundle {

    pub fn load_animations(&mut self, source_csv: &str) -> Result<(), Error> {
        fn parse(v: &str) -> f32 { str::parse::<f32>(v).unwrap_or(0.0) }

        split_csv::<6, _>(source_csv, |args| {
            let name = args[0];
            let frame_count = parse(args[1]);
            let left = parse(args[2]);
            let top = parse(args[3]);
            let right = parse(args[4]);
            let bottom = parse(args[5]);
            let sprite_width = (right-left) / frame_count;
            let sprite_height = bottom - top;
            let animation = AnimationBase {
                x: left,
                y: top,
                sprite_width,
                sprite_height,
                last_frame: (frame_count as u8) - 1,
            };

            let sprite_type = name.split('_').next().unwrap_or("");
            match sprite_type {
                "gobindynamite" => self.dynamite_goblin.set_animation_by_name(name, animation),
                "gobintorch" => self.torch_goblin.set_animation_by_name(name, animation),
                "sheep" => self.sheep.set_animation_by_name(name, animation),
                "pawn" => self.pawn.set_animation_by_name(name, animation),
                "archer" => self.archer.set_animation_by_name(name, animation),
                "warrior" => self.warrior.set_animation_by_name(name, animation),
                "death" => Some(()),
                _ => Some(())
            };
        });

        Ok(())
    }

}
