use crate::shared::AABB;

#[derive(Copy, Clone, Default)]
pub struct GuiBundle {
    // A solid color area
    pub solid: AABB,
    pub info_panel: AABB,
}

impl GuiBundle {

    pub fn load(&mut self, csv: &str) {
        fn parse(v: &str) -> f32 {
            str::parse::<f32>(v).unwrap_or(0.0)
        }

        crate::shared::split_csv::<6, _>(csv, |args| {
            let name = args[0];
            let left = parse(args[1]);
            let top = parse(args[2]);
            let right = parse(args[3]);
            let bottom = parse(args[4]);

            let aabb = match name {
                "solid" => &mut self.solid,
                "info_panel" => &mut self.info_panel,
                _ => {
                    return;
                }
            };

            *aabb = AABB { left, top, right, bottom };
        });
    }

}
