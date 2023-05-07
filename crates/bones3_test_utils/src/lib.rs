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
    /// Collects all events of the indicated type currently within the system
    /// and returns an iterator over all of them.
    ///
    /// Note that the events are still removed from the app, even the iterator
    /// is not used.
    fn collect_events<E: Event + Clone>(&mut self) -> Box<dyn Iterator<Item = E>>;
}

impl TestApp for App {
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
