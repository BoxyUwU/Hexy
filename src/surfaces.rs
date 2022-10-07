use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{hexmap::HexMap, simulation::MyTileData};

/// Runs a bunch of systems sequentailyl with no parallelism, flushing commands after each system runs
pub struct SimpleSchedule {
    systems: Vec<Box<dyn System<In = (), Out = ()> + Send + Sync>>,
}

impl SimpleSchedule {
    /// this fn will handle initializing `system`
    pub fn add_system(
        &mut self,
        mut system: Box<dyn System<In = (), Out = ()> + Send + Sync>,
        world: &mut World,
    ) -> &mut Self {
        system.initialize(world);
        self.systems.push(system);

        self
    }

    pub fn run_once(&mut self, world: &mut World) {
        for system in self.systems.iter_mut() {
            // change detection is definitely busted
            system.update_archetype_component_access(world);
            system.run((), world);
            system.apply_buffers(world);
        }
    }

    pub fn new() -> Self {
        Self { systems: vec![] }
    }
}

pub struct Surfaces {
    surfaces: Vec<(SimpleSchedule, World)>,
    existing_system_ctors:
        Vec<Box<dyn (Fn() -> Box<dyn System<In = (), Out = ()> + Send + Sync>) + Send + Sync>>,
}

impl Surfaces {
    pub fn new() -> Self {
        Self {
            surfaces: vec![],
            existing_system_ctors: vec![],
        }
    }

    pub fn new_surface(&mut self, mut world: World, tilemap: HexMap<MyTileData>) {
        assert!(!world.contains_resource::<HexMap<MyTileData>>());
        world.insert_resource(tilemap);

        let mut schedule = SimpleSchedule::new();
        for ctor in self.existing_system_ctors.iter_mut() {
            schedule.add_system(ctor(), &mut world);
        }

        self.surfaces.push((schedule, world));
    }

    // no support for `remove_surface` who knows if we'll need it

    pub fn push_system<Params>(
        &mut self,
        system: impl IntoSystem<(), (), Params> + Clone + Send + Sync + 'static,
    ) -> &mut Self {
        let system_ctor: Box<
            dyn Fn() -> Box<dyn System<In = (), Out = ()> + Send + Sync> + Send + Sync,
        > = Box::new(move || Box::new(IntoSystem::into_system(system.clone())));

        for (schedule, world) in self.surfaces.iter_mut() {
            schedule.add_system(system_ctor(), world);
        }
        self.existing_system_ctors.push(system_ctor);

        self
    }

    pub fn simulate_step(&mut self) {
        for (schedule, surface) in self.surfaces.iter_mut() {
            schedule.run_once(surface);
        }
    }
}

pub struct SelectedSurface(pub usize);

#[derive(SystemParam)]
pub struct CurrentHexMap<'w, 's> {
    selected: Res<'w, SelectedSurface>,
    surfaces: Res<'w, Surfaces>,
    #[system_param(ignore)]
    _p: PhantomData<&'s ()>,
}

impl CurrentHexMap<'_, '_> {
    pub fn hexmap(&self) -> &HexMap<MyTileData> {
        self.surfaces.surfaces[self.selected.0]
            .1
            .get_resource()
            .unwrap()
    }
}
