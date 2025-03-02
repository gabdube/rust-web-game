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
}

impl ResourcesBundle {

    pub fn load(&mut self, csv: &str) {
        fn parse(v: &str) -> f32 {
            str::parse::<f32>(v).unwrap_or(0.0)
        }

        crate::shared::split_csv::<5, _>(csv, |args| {
            let name = args[0];
            let left = parse(args[1]);
            let top = parse(args[2]);
            let right = parse(args[3]);
            let bottom = parse(args[4]);

            if let Some(base) = self.match_name(name) {
                base.aabb = AABB { left, top, right, bottom };
            }
        });
    }

    fn match_name(&mut self, name: &str) -> Option<&mut ResourceBase> {
        match name {
            "explosive_barrel" => Some(&mut self.explosive_barrel),
            "gold" => Some(&mut self.gold),
            "gold_noshadow" => Some(&mut self.gold_shadowless),
            "meat" => Some(&mut self.meat),
            "meat_noshadow" => Some(&mut self.meat_shadowless),
            "wood" => Some(&mut self.wood),
            "wood_noshadow" => Some(&mut self.wood_shadowless),
            _ => None,
        }
    }

}
