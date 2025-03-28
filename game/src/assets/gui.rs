use crate::shared::AABB;

#[derive(Copy, Clone, Default)]
pub struct GuiBundle {
    pub info_panel: AABB,
    pub pawn_portrait: AABB,
    pub warrior_portrait: AABB,
    pub archer_portrait: AABB,
    pub goblin_dynamite_portrait: AABB,
    pub goblin_torch_portrait: AABB,
    pub sheep_portrait: AABB,
    pub gold_icon: AABB,
    pub meat_icon: AABB,
    pub wood_icon: AABB,
}

impl GuiBundle {

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

            let aabb = match name {
                "info_panel" => &mut self.info_panel,
                "pawn_portrait" => &mut self.pawn_portrait,
                "warrior_portrait" => &mut self.warrior_portrait,
                "archer_portrait" => &mut self.archer_portrait,
                "gobindynamite_portrait" => &mut self.goblin_dynamite_portrait,
                "gobintorch_portrait" => &mut self.goblin_torch_portrait,
                "sheep_portrait" => &mut self.sheep_portrait,
                "gold_icon" => &mut self.gold_icon,
                "meat_icon" => &mut self.meat_icon,
                "wood_icon" => &mut self.wood_icon,
                _ => {
                    return;
                }
            };

            *aabb = AABB { left, top, right, bottom };
        });
    }

}
