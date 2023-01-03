//! The default plugin group for Bones Cubed.

use std::marker::PhantomData;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bones3_core::storage::BlockData;
use bones3_core::Bones3CorePlugin;

/// The core plugin for Bones Cubed.
///
/// This plugin group contains the following plugins:
/// * [`bones3_core::Bones3CorePlugin`]
#[derive(Default)]
pub struct Bones3Plugins<T>
where
    T: BlockData,
{
    /// Phantom data for T.
    _phantom: PhantomData<T>,
}

impl<T> PluginGroup for Bones3Plugins<T>
where
    T: BlockData,
{
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(Bones3CorePlugin::<T>::default())
    }
}
