mod terrain;
use terrain::Terrain;

mod extra_data;
pub use extra_data::*;

use crate::assets::{AnimationBase, StructureBase, Texture};
use crate::behaviour;
use crate::shared::{AABB, aabb, size, pos};
use crate::store::SaveAndLoad;
use crate::Position;


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WorldObjectType {
    Pawn = 0,
    Warrior,
    Archer,
    TorchGoblin,
    DynamiteGoblin,
    Sheep,
    Decoration,
    Structure,
    Resource,
    ResourceSpawn,
    Tree,
}

impl WorldObjectType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pawn => "Pawn",
            Self::Warrior => "Warrior",
            Self::Archer => "Archer",
            Self::TorchGoblin => "Torch Goblin",
            Self::DynamiteGoblin => "Dynamite Goblin",
            Self::Sheep => "Sheep",
            Self::Decoration => "Decoration",
            Self::Structure => "Structure",
            Self::Tree => "Tree",
            Self::Resource => "Resource",
            _ => ""
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WorldObject {
    pub id: u32,
    pub ty: WorldObjectType,
}

#[derive(Copy, Clone, Default)]
pub struct BaseAnimated {
    pub position: Position<f32>,
    pub animation: AnimationBase,
    pub current_frame: u8,
    pub selected: bool,
    pub flipped: bool,
    pub deleted: bool,
}

impl BaseAnimated {
    pub const fn aabb(&self) -> AABB {
        let mut position = self.position;
        let size = size(self.animation.sprite_width, self.animation.sprite_height);
        position.x -= size.width * 0.5;
        position.y -= size.height;
        aabb(position, size)
    }

    /// Marks this animation as "deleted".
    pub fn delete(&mut self) {
        self.position = pos(0.0, 0.0);
        self.animation = AnimationBase::default();
        self.selected = false;
        self.flipped = false;
        self.deleted = true;
    }
}

#[derive(Copy, Clone, Default)]
pub struct BaseStatic {
    pub position: Position<f32>,
    pub sprite: AABB,
    pub selected: bool,
    // pub deleted: bool,
}

impl BaseStatic {
    pub const fn aabb(&self) -> AABB {
        let mut position = self.position;
        let width = self.sprite.width();
        let height = self.sprite.height();
        position.x -= width * 0.5;
        position.y -= height;
        aabb(position, size(width, height))
    }

    // pub fn delete(&mut self) {
    //     self.position = pos(0.0, 0.0);
    //     self.sprite = AABB::default();
    //     self.selected = false;
    //     self.deleted = true;
    // }
}

#[derive(Copy, Clone)]
pub struct BaseProjectile {
    pub position: Position<f32>,
    pub sprite: AABB,
    pub rotation: f32,
    pub deleted: bool,
}


/// The game world data. Includes actors, terrain, and decorations
pub struct World {
    pub total_sprite_count: u32,

    pub static_resources_texture: Texture,
    pub units_texture: Texture,

    pub terrain: Terrain,

    pub pawns: Vec<BaseAnimated>,
    pub pawns_data: Vec<PawnData>,
    pub pawns_behaviour: Vec<behaviour::pawn::PawnBehaviour>,

    pub warriors: Vec<BaseAnimated>,
    pub warriors_behaviour: Vec<behaviour::warrior::WarriorBehaviour>,

    pub archers: Vec<BaseAnimated>,
    pub archers_behaviour: Vec<behaviour::archer::ArcherBehaviour>,

    pub torch_goblins: Vec<BaseAnimated>,
    pub tnt_goblins: Vec<BaseAnimated>,

    pub sheeps: Vec<BaseAnimated>,
    pub sheeps_data: Vec<SheepData>,
    pub sheep_behaviour: Vec<behaviour::sheep::SheepBehaviour>,

    pub structures: Vec<BaseStatic>,
    pub structures_data: Vec<StructureData>,

    pub resources: Vec<BaseStatic>,
    pub resources_data: Vec<ResourceData>,

    pub resources_spawn: Vec<BaseAnimated>,
    pub resources_spawn_behaviour: Vec<behaviour::spawn_resources::SpawnResourceBehaviour>,
    
    pub trees: Vec<BaseAnimated>,
    pub trees_data: Vec<TreeData>,

    pub arrows: Vec<BaseProjectile>,
    pub arrows_data: Vec<ArrowData>,

    pub decorations: Vec<BaseStatic>,

    pub selected: Vec<WorldObject>
}

impl World {

    /// The total number of sprites in the world
    /// Used to preallocate the sprite buffer in output
    pub fn total_sprites(&self) -> usize {
        self.total_sprite_count as usize
    }

    /// The total number of projectile sprites in the world
    /// Used to preallocate the sprite buffer in output
    pub fn total_projectile_sprites(&self) -> usize {
        self.arrows.len()
    }

    pub fn reset(&mut self) {
        self.pawns.clear();
        self.pawns_data.clear();
        self.pawns_behaviour.clear();

        self.warriors.clear();
        self.warriors_behaviour.clear();

        self.archers.clear();
        self.archers_behaviour.clear();

        self.torch_goblins.clear();
        self.tnt_goblins.clear();

        self.sheeps.clear();
        self.sheeps_data.clear();
        self.sheep_behaviour.clear();

        self.decorations.clear();
        
        self.structures.clear();
        self.structures_data.clear();

        self.resources.clear();
        self.resources_data.clear();

        self.resources_spawn.clear();
        self.resources_spawn_behaviour.clear();

        self.trees.clear();
        self.trees_data.clear();

        self.arrows.clear();
        self.arrows_data.clear();

        self.selected.clear();
        self.terrain.reset();
    }

    pub fn init_terrain(&mut self, width: u32, height: u32) {
        self.terrain.init_terrain(width, height);
    }

    pub fn create_pawn(&mut self, position: Position<f32>) -> usize {
        self.total_sprite_count += 1;
        self.pawns_data.push(PawnData::default());
        self.pawns_behaviour.push(behaviour::pawn::PawnBehaviour::idle());
        self.pawns.push(BaseAnimated { position, ..Default::default()});
        self.pawns.len() - 1
    }

    pub fn create_warrior(&mut self, position: Position<f32>) -> usize {
        self.total_sprite_count += 1;
        self.warriors_behaviour.push(behaviour::warrior::WarriorBehaviour::idle());
        self.warriors.push(BaseAnimated { position, ..Default::default()});
        self.warriors.len() - 1
    }

    pub fn create_archer(&mut self, position: Position<f32>) -> usize {
        self.total_sprite_count += 1;
        self.archers_behaviour.push(behaviour::archer::ArcherBehaviour::idle());
        self.archers.push(BaseAnimated { position, ..Default::default()});
        self.archers.len() - 1
    }

    pub fn create_sheep(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
        self.total_sprite_count += 1;
        self.sheeps_data.push(SheepData::default());
        self.sheep_behaviour.push(behaviour::sheep::SheepBehaviour::idle());
        Self::create_inner_actor(&mut self.sheeps, position, animation)
    }

    pub fn create_tree(&mut self, position: Position<f32>, animation: &AnimationBase) -> usize {
        self.total_sprite_count += 1;
        self.trees_data.push(TreeData::default());
        Self::create_inner_actor(&mut self.trees, position, animation)
    }

    pub fn create_gold_mine(&mut self, position: Position<f32>, sprite: StructureBase) {
        self.total_sprite_count += 1;
        self.structures.push(BaseStatic { position, sprite: sprite.aabb, selected: false });
        self.structures_data.push(StructureData::GoldMine(Default::default()));
    }

    pub fn create_castle(&mut self, position: Position<f32>, sprite: StructureBase) -> usize {
        self.total_sprite_count += 1;
        self.structures.push(BaseStatic { position, sprite: sprite.aabb, selected: false });
        self.structures_data.push(StructureData::Castle(StructureCastleData { hp: 0, building: true, destroyed: false }));
        self.structures.len() - 1
    }

    pub fn create_tower(&mut self, position: Position<f32>, sprite: StructureBase) -> usize {
        self.total_sprite_count += 1;
        self.structures.push(BaseStatic { position, sprite: sprite.aabb, selected: false });
        self.structures_data.push(StructureData::Tower(StructureTowerData { hp: 0, building: true, destroyed: false }));
        self.structures.len() - 1
    }

    pub fn create_house(&mut self, position: Position<f32>, sprite: StructureBase) -> usize {
        self.total_sprite_count += 1;
        self.structures.push(BaseStatic { position, sprite: sprite.aabb, selected: false });
        self.structures_data.push(StructureData::House(StructureHouseData { hp: 0, building: true, destroyed: false }));
        self.structures.len() - 1
    }

    pub fn create_resource_spawn(&mut self, position: Position<f32>, animation: &AnimationBase, resource_type: ResourceType) -> usize {
        self.total_sprite_count += 1;
        self.resources_spawn_behaviour.push(behaviour::spawn_resources::SpawnResourceBehaviour::spawn(resource_type));
        Self::create_inner_actor(&mut self.resources_spawn, position, animation)
    }

    pub fn create_resource(&mut self, position: Position<f32>, sprite: AABB, resource_data: ResourceData) -> usize {
        self.total_sprite_count += 1;
        self.resources.push(BaseStatic { position, sprite, selected: false });
        self.resources_data.push(resource_data);
        self.resources.len() - 1
    }

    pub fn object_at(&self, position: Position<f32>) -> Option<WorldObject> {
        fn animated_at(world: &World, position: Position<f32>, out: &mut Option<WorldObject>, y_out: &mut f32) {
            let types = [
                WorldObjectType::Pawn,
                WorldObjectType::Warrior,
                WorldObjectType::Archer,
                WorldObjectType::TorchGoblin,
                WorldObjectType::DynamiteGoblin,
                WorldObjectType::Sheep,
                WorldObjectType::ResourceSpawn,
                WorldObjectType::Tree,
            ];
            let groups: [&[BaseAnimated]; 8] = [
                &world.pawns,
                &world.warriors,
                &world.archers,
                &world.torch_goblins,
                &world.tnt_goblins,
                &world.sheeps,
                &world.resources_spawn,
                &world.trees,
            ];
    
            let mut obj = None;
            let mut y = *y_out;
    
            for (group, ty) in groups.into_iter().zip(types) {
                for (id, actor) in group.iter().enumerate() {
                    let aabb = actor.aabb();
                    if aabb.point_inside(position) {
                        if aabb.bottom > y {
                            obj = Some(WorldObject { id: id as u32, ty });
                            y = aabb.bottom;
                        }
                    }
                }
            }
    
            if obj.is_some() {
                *out = obj;
                *y_out = y;
            }
        }
    
        fn other_at(world: &World, position: Position<f32>, out: &mut Option<WorldObject>, y_out: &mut f32) {
            let types = [
                WorldObjectType::Structure,
                WorldObjectType::Resource,
            ];
            let groups = [
                &world.structures,
                &world.resources,
            ];
    
            let mut obj = None;
            let mut y = *y_out;
    
            for (group, ty) in groups.into_iter().zip(types) {
                for (id, resource) in group.iter().enumerate() {
                    let aabb = resource.aabb();
                    if aabb.point_inside(position) {
                        if aabb.bottom > y {
                            obj = Some(WorldObject { id: id as u32, ty });
                            y = aabb.bottom;
                        }
                    }
                }
            }
    
            if obj.is_some() {
                *out = obj;
                *y_out = y;
            }
        }
        
        let mut obj = None;
        let mut y = -10000.0;

        animated_at(self, position, &mut obj, &mut y);
        other_at(self, position, &mut obj, &mut y);

        obj
    }

    pub fn get_actor_mut<'a>(&'a mut self, obj: WorldObject) -> Option<&'a mut BaseAnimated> {
        let objects = match obj.ty {
            WorldObjectType::Pawn => &mut self.pawns,
            WorldObjectType::Warrior => &mut self.warriors,
            WorldObjectType::Archer => &mut self.archers,
            WorldObjectType::TorchGoblin => &mut self.torch_goblins,
            WorldObjectType::DynamiteGoblin => &mut self.tnt_goblins,
            WorldObjectType::Sheep => &mut self.sheeps,
            WorldObjectType::Tree => &mut self.trees,
            _ => { return None }
        };

        objects.get_mut(obj.id as usize)
    }

    pub fn get_static_mut<'a>(&'a mut self, obj: WorldObject) -> Option<&'a mut BaseStatic> {
        let objects = match obj.ty {
            WorldObjectType::Decoration => &mut self.decorations,
            WorldObjectType::Structure => &mut self.structures,
            WorldObjectType::Resource => &mut self.resources,
            _ => { return None }
        };

        objects.get_mut(obj.id as usize)
    }

    pub fn set_object_selected(&mut self, obj: WorldObject, selected: bool) {
        let mut add = false;
        let mut remove = false;

        if let Some(actor) = self.get_actor_mut(obj) {
            add = !actor.selected && selected;
            remove = actor.selected && !selected;
            actor.selected = selected;
        } else if let Some(statiq) = self.get_static_mut(obj) {
            add = !statiq.selected && selected;
            remove = statiq.selected && !selected;
            statiq.selected = selected;
        }

        if add {
            self.selected.push(obj);
        } else if remove {
            if let Some(index) = self.selected.iter().position(|&obj2| obj == obj2 ) {
                self.selected.swap_remove(index);
            }
        }
    }

    fn create_inner_actor(
        base: &mut Vec<BaseAnimated>,
        position: Position<f32>,
        animation: &AnimationBase
    ) -> usize {
        let index = base.len();
        base.push(BaseAnimated { position, animation: *animation, current_frame: 0, ..Default::default()});
        return index
    }

}

impl SaveAndLoad for World {

    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.pawns);
        writer.write_slice(&self.pawns_data);
        writer.write_slice(&self.pawns_behaviour);

        writer.write_slice(&self.warriors);
        writer.write_slice(&self.warriors_behaviour);

        writer.write_slice(&self.archers);
        writer.write_slice(&self.archers_behaviour);

        writer.write_slice(&self.torch_goblins);
        writer.write_slice(&self.tnt_goblins);

        writer.write_slice(&self.sheeps);
        writer.write_slice(&self.sheeps_data);
        writer.save_slice(&self.sheep_behaviour);

        writer.write_slice(&self.decorations);

        writer.write_slice(&self.structures);
        writer.write_slice(&self.structures_data);
        
        writer.write_slice(&self.resources);
        writer.write_slice(&self.resources_data);
        writer.write_slice(&self.resources_spawn);
        writer.write_slice(&self.resources_spawn_behaviour);

        writer.write_slice(&self.trees);
        writer.save_slice(&self.trees_data);

        writer.write_slice(&self.arrows);
        writer.write_slice(&self.arrows_data);

        writer.write_slice(&self.selected);

        writer.write(&self.static_resources_texture);
        writer.write(&self.units_texture);
        writer.write_u32(self.total_sprite_count);

        writer.save(&self.terrain);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let pawns = reader.read_vec();
        let pawns_data = reader.read_vec();
        let pawns_behaviour = reader.read_vec();

        let warriors = reader.read_vec();
        let warriors_behaviour = reader.read_vec();

        let archers = reader.read_vec();
        let archers_behaviour = reader.read_vec();

        let torch_goblins = reader.read_vec();
        let tnt_goblins = reader.read_vec();

        let sheeps = reader.read_vec();
        let sheeps_data = reader.read_vec();
        let sheep_behaviour = reader.load_vec();

        let decorations = reader.read_vec();

        let structures = reader.read_vec();
        let structures_data = reader.read_vec();
        
        let resources = reader.read_vec();
        let resources_data = reader.read_vec();
        let resources_spawn = reader.read_vec();
        let resources_spawn_behaviour = reader.read_vec();

        let trees = reader.read_vec();
        let trees_data = reader.load_vec();

        let arrows = reader.read_vec();
        let arrows_data = reader.read_vec();

        let selected = reader.read_vec();

        let static_resources_texture = reader.read();
        let units_texture = reader.read();
        let total_sprite_count = reader.read_u32();

        let terrain = reader.load();

        World {
            total_sprite_count,
            static_resources_texture,
            units_texture,

            terrain,

            pawns,
            pawns_data,
            pawns_behaviour,

            warriors,
            warriors_behaviour,

            archers,
            archers_behaviour,

            torch_goblins,
            tnt_goblins,

            sheeps,
            sheeps_data,
            sheep_behaviour,

            structures,
            structures_data,

            resources,
            resources_data,
            resources_spawn,
            resources_spawn_behaviour,

            trees,
            trees_data,

            arrows,
            arrows_data,

            decorations,

            selected,
        }
    }

}

impl Default for World {
    fn default() -> Self {
        World {
            total_sprite_count: 0,
            static_resources_texture: Texture { id: 0 },
            units_texture: Texture { id: 0 },

            terrain: Terrain::default(),
    
            pawns: Vec::with_capacity(16),
            pawns_data: Vec::with_capacity(16),
            pawns_behaviour: Vec::with_capacity(16),

            warriors: Vec::with_capacity(16),
            warriors_behaviour: Vec::with_capacity(16),

            archers: Vec::with_capacity(16),
            archers_behaviour: Vec::with_capacity(16),

            torch_goblins: Vec::with_capacity(16),
            tnt_goblins: Vec::with_capacity(16),

            sheeps: Vec::with_capacity(16),
            sheeps_data: Vec::with_capacity(16),
            sheep_behaviour: Vec::with_capacity(16),
    
            structures: Vec::with_capacity(16),
            structures_data: Vec::with_capacity(16),
            
            resources: Vec::with_capacity(32),
            resources_data: Vec::with_capacity(32),
            resources_spawn: Vec::with_capacity(16),
            resources_spawn_behaviour: Vec::with_capacity(16),

            trees: Vec::with_capacity(16),
            trees_data: Vec::with_capacity(16),

            arrows: Vec::with_capacity(16),
            arrows_data: Vec::with_capacity(16),

            decorations: Vec::with_capacity(16),

            selected: Vec::with_capacity(8),
        }
    }
}

impl SaveAndLoad for WorldObject {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_u32(self.ty as u32);
        writer.write_u32(self.id);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let ty = match reader.read_u32() {
            0 => WorldObjectType::Pawn,
            1 => WorldObjectType::Warrior,
            2 => WorldObjectType::Archer,
            3 => WorldObjectType::TorchGoblin,
            4 => WorldObjectType::DynamiteGoblin,
            5 => WorldObjectType::Sheep,
            6 => WorldObjectType::Decoration,
            7 => WorldObjectType::Structure,
            8 => WorldObjectType::Resource, 
            9 => WorldObjectType::Tree,
            _ => WorldObjectType::Pawn,
        };

        let id = reader.read_u32();

        WorldObject {
            id,
            ty,
        }
    }
}