use crate::shared::AABB;

#[allow(dead_code)]
#[derive(Copy, Clone, Default)]
pub struct StructuresBundle {
    pub knights_castle: AABB,
    pub knights_castle_construction: AABB,
    pub knights_castle_destroyed: AABB,
    pub knights_tower: AABB,
    pub knights_tower_construction: AABB,
    pub knights_tower_destroyed: AABB,
    pub knights_house: AABB,
    pub knights_house_construction: AABB,
    pub knights_house_destroyed: AABB,
    pub goblin_house: AABB,
    pub goblin_house_destroyed: AABB,
    pub gold_mine: AABB,
    pub gold_mine_destroyed: AABB,
    pub gold_mine_inactive: AABB,
}

impl StructuresBundle {

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
                *base = AABB { left, top, right, bottom };
            }
        });
    }

    fn match_name(&mut self, name: &str) -> Option<&mut AABB> {
        match name {
            "knight_castle" => Some(&mut self.knights_castle),
            "knight_castle_construction" => Some(&mut self.knights_castle_construction),
            "knight_castle_destroyed" => Some(&mut self.knights_castle_destroyed),
            "knight_tower" => Some(&mut self.knights_tower),
            "knight_tower_construction" => Some(&mut self.knights_tower_construction),
            "knight_tower_destroyed" => Some(&mut self.knights_tower_destroyed),
            "knight_house" => Some(&mut self.knights_house),
            "knight_house_construction" => Some(&mut self.knights_house_construction),
            "knight_house_destroyed" => Some(&mut self.knights_house_destroyed),
            "goblin_house" =>  Some(&mut self.goblin_house),
            "goblin_house_destroyed" =>  Some(&mut self.goblin_house_destroyed),
            "gold_mine" =>  Some(&mut self.gold_mine),
            "gold_mine_inactive" =>  Some(&mut self.gold_mine_inactive),
            "gold_mine_destroyed" =>  Some(&mut self.gold_mine_destroyed),
            _ => None,
        }
    }

}

