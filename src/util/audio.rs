use std::thread::{self, JoinHandle, ThreadId};

use miette::Diagnostic;
use thiserror::Error;

pub struct Audio {
    pw_thread: JoinHandle<()>,
}

impl Audio {
    pub fn new() -> Result<Self, AudioError> {
        let pw_thread = thread::spawn(move || {
            let mainloop = MainLoop::new(None)?;
            let context = Context::new(&mainloop)?;
            let core = context.connect(None)?;
            let registry = core.get_registry()?;

            // Register a callback to the `global` event on the registry, which notifies of any new global objects
            // appearing on the remote.
            // The callback will only get called as long as we keep the returned listener alive.
            let _listener = registry
                .add_listener_local()
                .global(|global| println!("New global: {:?}", global))
                .register();

            // Calling the `destroy_global` method on the registry will destroy the object with the specified id on the remote.
            // We don't have a specific object to destroy now, so this is commented out.
            // registry.destroy_global(313).into_result()?;

            mainloop.run();
        });

        Ok(Self { pw_thread })
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum AudioError {}
