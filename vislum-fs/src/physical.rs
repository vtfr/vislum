use std::{
    path::PathBuf,
    time::Duration,
};

use crossbeam::channel::Sender;
use notify_debouncer_full::{
    DebounceEventResult,
    Debouncer,
    RecommendedCache,
    notify::{EventKind, RecommendedWatcher, RecursiveMode},
};

use crate::{FileEvent, FileEventType, VirtualNamespace, VirtualPath, Watcher};

pub struct PhysicalFsWatcher {
    #[allow(dead_code)]
    debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

impl Watcher for PhysicalFsWatcher {
    fn new(
        namespace: VirtualNamespace,
        root: PathBuf,
        event_tx: Sender<FileEvent>,
    ) -> Box<dyn Watcher> {
        let mut debouncer = notify_debouncer_full::new_debouncer(
            Duration::from_secs(1),
            None,
            move |events: DebounceEventResult| {
                let Ok(events) = events else {
                    return;
                };

                for event in events {
                    // Filter only modify, create, and remove events.
                    let kind = match event.event.kind {
                        EventKind::Create(_) => FileEventType::Created,
                        EventKind::Modify(_) => FileEventType::Modified,
                        EventKind::Remove(_) => FileEventType::Removed,
                        _ => return,
                    };

                    for path in event.event.paths {
                        // Convert the filesystem path to a virtual path.
                        let virtual_path = VirtualPath::new(namespace, &*path);
                        let event = FileEvent {
                            path: virtual_path,
                            event_type: kind,
                        };

                        // Send the event to the channel.
                        let _ = event_tx.send(event);
                    }
                }
            },
        )
        .unwrap();

        debouncer.watch(&root, RecursiveMode::Recursive).unwrap();

        Box::new(Self { debouncer })
    }
}

