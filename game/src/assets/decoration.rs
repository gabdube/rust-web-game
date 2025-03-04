use crate::shared::AABB;


#[derive(Copy, Clone, Default)]
pub struct DecorationBase {
    pub aabb: AABB
}

#[allow(dead_code)]
#[derive(Copy, Clone, Default)]
pub struct DecorationBundle {
    pub sign1: DecorationBase,
    pub sign2: DecorationBase,
    pub sign3: DecorationBase,
    pub shroom_big: DecorationBase,
    pub shroom_med: DecorationBase,
    pub shroom_sml: DecorationBase,
    pub rock_big: DecorationBase,
    pub rock_med: DecorationBase,
    pub rock_sml: DecorationBase,
    pub bush_big: DecorationBase,
    pub bush_med: DecorationBase,
    pub bush_sml: DecorationBase,
    pub plant_med: DecorationBase,
    pub plant_sml: DecorationBase,
    pub pumpkin_med: DecorationBase,
    pub pumpkin_sml: DecorationBase,
    pub bone1: DecorationBase,
    pub bone2: DecorationBase,
    pub tree_stump: DecorationBase,
}

impl DecorationBundle {

    pub fn load(&mut self, csv: &str) {
        fn parse(v: &str) -> f32 {
            str::parse::<f32>(v).unwrap_or(0.0)
        }

        crate::shared::split_csv::<6, _>(csv, |args| {
            let name = args[0];
            let left = parse(args[2]);
            let top = parse(args[3]);
            let right = parse(args[4]);
            let bottom = parse(args[5]);

            if let Some(base) = self.match_name(name) {
                base.aabb = AABB { left, top, right, bottom };
            }
        });
    }

    fn match_name(&mut self, name: &str) -> Option<&mut DecorationBase> {
        match name {
            "sign_1" => Some(&mut self.sign1),
            "sign_2" => Some(&mut self.sign2),
            "sign_3" => Some(&mut self.sign3),
            "shroom_big" => Some(&mut self.shroom_big),
            "shroom_medium" => Some(&mut self.shroom_med),
            "shroom_small" => Some(&mut self.shroom_sml),
            "rock_big" => Some(&mut self.rock_big),
            "rock_medium" => Some(&mut self.rock_med),
            "rock_small" => Some(&mut self.rock_sml),
            "bush_big" => Some(&mut self.bush_big),
            "bush_medium" => Some(&mut self.bush_med),
            "bush_small" => Some(&mut self.bush_sml),
            "plant_medium" => Some(&mut self.plant_med),
            "plant_small" => Some(&mut self.plant_sml),
            "pumpkin_medium" => Some(&mut self.pumpkin_med),
            "pumpkin_small" => Some(&mut self.pumpkin_sml),
            "bone1" => Some(&mut self.bone1),
            "bone2" => Some(&mut self.bone2),
            "tree_stump" => Some(&mut self.tree_stump),
            _ => None,
        }
    }

}
