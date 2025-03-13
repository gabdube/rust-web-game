use crate::assets::animations::AnimationBase;
use crate::shared::AABB;


#[derive(Copy, Clone, Default)]
pub struct ResourceBase {
    pub aabb: AABB
}

#[allow(dead_code)]
#[derive(Copy, Clone, Default)]
pub struct ResourcesBundle {
    pub explosive_barrel: ResourceBase,
    pub gold: ResourceBase,
    pub gold_shadowless: ResourceBase,
    pub meat: ResourceBase,
    pub meat_shadowless: ResourceBase,
    pub wood: ResourceBase,
    pub wood_shadowless: ResourceBase,
    pub tree_stump: ResourceBase,

    pub tree_idle: AnimationBase,
    pub tree_cut: AnimationBase,

    pub gold_spawn: AnimationBase,
    pub meat_spawn: AnimationBase,
    pub wood_spawn: AnimationBase,
}

impl ResourcesBundle {

    pub fn load(&mut self, csv: &str) {
        fn parse(v: &str) -> f32 {
            str::parse::<f32>(v).unwrap_or(0.0)
        }

        crate::shared::split_csv::<6, _>(csv, |args| {
            let name = args[0];
            let frame_count = str::parse::<u32>(args[1]).unwrap_or(1);
            let left = parse(args[2]);
            let top = parse(args[3]);
            let right = parse(args[4]);
            let bottom = parse(args[5]);

            if frame_count == 1 {
                if let Some(base) = self.match_static_name(name) {
                    base.aabb = AABB { left, top, right, bottom };
                }
            } else {
                if let Some(base) = self.match_animated_name(name) {
                    let sprite_width = (right-left) / (frame_count as f32);
                    let sprite_height = bottom - top;
                    *base = AnimationBase {
                        x: left,
                        y: top,
                        sprite_width,
                        sprite_height,
                        last_frame: (frame_count as u8) - 1,
                    };
                }
            }
        });
    }

    fn match_static_name(&mut self, name: &str) -> Option<&mut ResourceBase> {
        match name {
            "explosive_barrel" => Some(&mut self.explosive_barrel),
            "gold" => Some(&mut self.gold),
            "gold_noshadow" => Some(&mut self.gold_shadowless),
            "meat" => Some(&mut self.meat),
            "meat_noshadow" => Some(&mut self.meat_shadowless),
            "wood" => Some(&mut self.wood),
            "wood_noshadow" => Some(&mut self.wood_shadowless),
            "tree_stump" => Some(&mut self.tree_stump),
            _ => None,
        }
    }

    fn match_animated_name(&mut self, name: &str) -> Option<&mut AnimationBase> {
        match name {
            "tree_idle" => Some(&mut self.tree_idle),
            "tree_cut" => Some(&mut self.tree_cut),
            "gold_spawn" => Some(&mut self.gold_spawn),
            "meat_spawn" => Some(&mut self.meat_spawn),
            "wood_spawn" => Some(&mut self.wood_spawn),
            _ => None
        }
    }

}
