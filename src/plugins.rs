//! The default plugin group for Bones Cubed.

use std::marker::PhantomData;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bones3_core::prelude::*;
#[cfg(feature = "meshing")]
use bones3_remesh::prelude::*;

/// A generator for initializing a block data trait that extends the indicated
/// traits based off of the provided enabled and disabled feature flags.
macro_rules! full_block_data_condition {
    ( [$($include:literal),*] - [$($exclude:literal),*] => $($data:ident),* ) => {
        /// A procedurally generated trait that includes all of the traits of a
        /// block data struct that must be defined in order for the Bones Cubed
        /// plugin to be used. These traits are enabled or disabled based on the
        /// currently active feature flags.
        #[cfg(all(
            $(feature = $include,)*
            $(not(feature = $exclude),)*
        ))]
        pub trait FullBlockData: BlockData $(+ $data)* {}
        #[cfg(all(
            $(feature = $include,)*
            $(not(feature = $exclude),)*
        ))]
        impl<T> FullBlockData for T where T: BlockData $(+ $data)* {}
    };
}

full_block_data_condition!([] - ["meshing"] => );
full_block_data_condition!(["meshing"] - [] => BlockShape);

/// The core plugin for Bones Cubed.
///
/// This plugin group contains the following plugins:
/// * [`bones3_core::Bones3CorePlugin`]
#[derive(Default)]
pub struct Bones3Plugins<T>
where
    T: FullBlockData,
{
    /// Phantom data for T.
    _phantom: PhantomData<T>,
}

impl<T> PluginGroup for Bones3Plugins<T>
where
    T: FullBlockData,
{
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>().add(Bones3CorePlugin::<T>::default());

        #[cfg(feature = "meshing")]
        {
            group = group.add(Bones3RemeshPlugin::<T>::default());
        }

        group
    }
}
