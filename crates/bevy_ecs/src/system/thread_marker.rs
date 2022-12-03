use fragile::Fragile;

use crate::prelude::World;
use crate::{self as bevy_ecs, prelude::Component};

use super::{Resource, SystemMeta, SystemParam, SystemParamFetch, SystemParamState};

pub struct MainThread;
impl SystemParam for MainThread {
    type Fetch = MainThreadState;
}

pub struct MainThreadState;

// SAFETY: this impl defers to `MainThreadState`, which initializes
// and validates the correct world access
unsafe impl SystemParamState for MainThreadState {
    fn init(_world: &mut World, system_meta: &mut SystemMeta) -> Self {
        system_meta.set_non_send();
        MainThreadState
    }
}

impl<'w, 's> SystemParamFetch<'w, 's> for MainThreadState {
    type Item = MainThread;

    #[inline]
    unsafe fn get_param(
        _state: &'s mut Self,
        _system_meta: &SystemMeta,
        _world: &'w World,
        _change_tick: u32,
    ) -> Self::Item {
        MainThread
    }
}

#[derive(Resource, Component)]
pub struct NonSend<T>(Fragile<T>);

impl<T> NonSend<T> {
    pub fn new(value: T) -> Self {
        NonSend(Fragile::new(value))
    }

    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }

    pub fn get(&self) -> &T {
        self.0.get()
    }

    // this takes an &mut self to trigger change detection when we get a mutable value out of the tls
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }
}

// SAFETY: pretty sure this is safe as ThreadLocal just wraps a usize and a phantom data
// and the usize is only written to on the call to ThreadLocal::new()
unsafe impl<T> Send for NonSend<T> {}
// SAFETY: pretty sure this is safe as ThreadLocal just wraps a usize and a phantom data
// and the usize is only written to on the call to ThreadLocal::new()
unsafe impl<T> Sync for NonSend<T> {}
