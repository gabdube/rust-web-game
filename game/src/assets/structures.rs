use crate::{error::Error, shared::AABB};

#[derive(Copy, Clone, Default)]
pub struct StructureBase {
    pub aabb: AABB
}

#[allow(dead_code)]
#[derive(Copy, Clone, Default)]
pub struct StructuresBundle {
    pub knights_castle: StructureBase,
    pub knights_castle_construction: StructureBase,
    pub knights_castle_destroyed: StructureBase,
    pub knights_tower: StructureBase,
    pub knights_tower_construction: StructureBase,
    pub knights_tower_destroyed: StructureBase,
    pub knights_house: StructureBase,
    pub knights_house_construction: StructureBase,
    pub knights_house_destroyed: StructureBase,
    pub goblin_house: StructureBase,
    pub goblin_house_destroyed: StructureBase,
    pub gold_mine: StructureBase,
    pub gold_mine_destroyed: StructureBase,
    pub gold_mine_inactive: StructureBase,
}

impl StructuresBundle {

    pub fn load(&mut self, csv: &str) -> Result<(), Error> {
        fn parse(v: &str) -> f32 {
            str::parse::<f32>(v).unwrap_or(0.0)
        }

        crate::shared::split_csv(csv, |args| {
            let name = args[0];
            let left = parse(args[1]);
            let top = parse(args[2]);
            let right = parse(args[3]);
            let bottom = parse(args[4]);

            if let Some(base) = self.match_name(name) {
                base.aabb = AABB { left, top, right, bottom };
            }
        });

        dbg!("{:?}", self.knights_castle.aabb);

        Ok(())
    }

    fn match_name(&mut self, name: &str) -> Option<&mut StructureBase> {
        match name {
            "knight_castle" => Some(&mut self.knights_castle),
            "knight_castle_construction" => Some(&mut self.knights_castle_construction),
            "knight_castle_destroyed" => Some(&mut self.knights_castle_destroyed),
            "knight_tower" => Some(&mut self.knights_tower),
            "knight_tower_construction" => Some(&mut self.knights_tower_construction),
            "knight_tower_destroyed" => Some(&mut self.knights_tower_destroyed),
            "knight_house" => Some(&mut self.knights_house),
            "knight_house_construction" => Some(&mut self.knights_house_construction),
            "knight_house_destroyed" => Some(&mut self.knights_castle_destroyed),
            "goblin_house" =>  Some(&mut self.goblin_house),
            "goblin_house_destroyed" =>  Some(&mut self.goblin_house_destroyed),
            "gold_mine" =>  Some(&mut self.gold_mine),
            "gold_mine_inactive" =>  Some(&mut self.gold_mine_inactive),
            "gold_mine_destroyed" =>  Some(&mut self.gold_mine_destroyed),
            _ => None,
        }
    }

}

