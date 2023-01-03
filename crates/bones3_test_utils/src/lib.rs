//! This cargo crate contains a set of useful unit testing utilities, designed
//! for the purpose of reducing boilerplate code when writing unit tests for
//! Bevy. This module focuses primarily on test cleanliness rather than
//! performance. As such, it is not recommended to use this library in a
//! production setting. This crate is intended to be used as a development
//! dependency only.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]

use bevy::ecs::event::Event;
use bevy::prelude::*;

/// An extension for the standard Bevy app that adds more unit test helper
/// functions.
pub trait TestApp {
    /// Causes the provided system to be executed once, immediately on the
    /// current thread.
    ///
    /// The system is not added to the app.
    fn run_system_once<Params>(&mut self, system: impl IntoSystemDescriptor<Params>);

    /// Causes the provided system set to be executed once, immediately, on
    /// parallel threads. Any queued commands are not executed until all systems
    /// within the list have been executed, as if it were a standard frame
    /// updating containing only the indicated system set.
    ///
    /// The system set is not added to the app.
    fn run_system_set_once(&mut self, system: SystemSet);

    /// Collects all events of the indicated type currently within the system
    /// and returns an iterator over all of them.
    ///
    /// Note that the events are still removed from the app, even the iterator
    /// is not used.
    fn collect_events<E: Event + Clone>(&mut self) -> Box<dyn Iterator<Item = E>>;
}

impl TestApp for App {
    fn run_system_once<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) {
        SystemStage::single(system).run(&mut self.world);
    }

    fn run_system_set_once(&mut self, systems: SystemSet) {
        SystemStage::parallel()
            .with_system_set(systems)
            .run(&mut self.world);
    }

    fn collect_events<E: Event + Clone>(&mut self) -> Box<dyn Iterator<Item = E>> {
        let event_res = self.world.resource::<Events<E>>();
        let mut event_reader = event_res.get_reader();
        Box::new(
            event_reader
                .iter(event_res)
                .map(|e| (*e).clone())
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}
