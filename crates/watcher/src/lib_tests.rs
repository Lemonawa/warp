use std::collections::HashSet;
use std::path::PathBuf;

use crate::BulkFilesystemWatcherEvent;

// --- BulkFilesystemWatcherEvent::is_empty ---

#[test]
fn default_event_is_empty() {
    let event = BulkFilesystemWatcherEvent::default();
    assert!(event.is_empty());
}

#[test]
fn event_with_added_is_not_empty() {
    let mut event = BulkFilesystemWatcherEvent::default();
    event.added.insert(PathBuf::from("/tmp/file.txt"));
    assert!(!event.is_empty());
}

#[test]
fn event_with_modified_is_not_empty() {
    let mut event = BulkFilesystemWatcherEvent::default();
    event.modified.insert(PathBuf::from("/tmp/file.txt"));
    assert!(!event.is_empty());
}

#[test]
fn event_with_deleted_is_not_empty() {
    let mut event = BulkFilesystemWatcherEvent::default();
    event.deleted.insert(PathBuf::from("/tmp/file.txt"));
    assert!(!event.is_empty());
}

#[test]
fn event_with_moved_is_not_empty() {
    let mut event = BulkFilesystemWatcherEvent::default();
    event
        .moved
        .insert(PathBuf::from("/tmp/new.txt"), PathBuf::from("/tmp/old.txt"));
    assert!(!event.is_empty());
}

// --- added_or_updated_iter ---

#[test]
fn added_or_updated_iter_returns_added_and_modified() {
    let mut event = BulkFilesystemWatcherEvent::default();
    event.added.insert(PathBuf::from("/a"));
    event.modified.insert(PathBuf::from("/b"));
    event.deleted.insert(PathBuf::from("/c"));

    let result: HashSet<&PathBuf> = event.added_or_updated_iter().collect();
    assert!(result.contains(&PathBuf::from("/a")));
    assert!(result.contains(&PathBuf::from("/b")));
    assert!(!result.contains(&PathBuf::from("/c")));
    assert_eq!(result.len(), 2);
}

#[test]
fn added_or_updated_iter_empty_event() {
    let event = BulkFilesystemWatcherEvent::default();
    assert_eq!(event.added_or_updated_iter().count(), 0);
}

// --- added_or_updated_set ---

#[test]
fn added_or_updated_set_returns_owned_set() {
    let mut event = BulkFilesystemWatcherEvent::default();
    event.added.insert(PathBuf::from("/a"));
    event.modified.insert(PathBuf::from("/b"));

    let set = event.added_or_updated_set();
    assert!(set.contains(&PathBuf::from("/a")));
    assert!(set.contains(&PathBuf::from("/b")));
    assert_eq!(set.len(), 2);
}

#[test]
fn added_or_updated_set_deduplicates() {
    let mut event = BulkFilesystemWatcherEvent::default();
    let path = PathBuf::from("/a");
    event.added.insert(path.clone());
    event.modified.insert(path);

    let set = event.added_or_updated_set();
    assert_eq!(set.len(), 1);
}

// --- Clone ---

#[test]
fn event_clone_preserves_all_fields() {
    let mut event = BulkFilesystemWatcherEvent::default();
    event.added.insert(PathBuf::from("/added"));
    event.modified.insert(PathBuf::from("/modified"));
    event.deleted.insert(PathBuf::from("/deleted"));
    event
        .moved
        .insert(PathBuf::from("/new_name"), PathBuf::from("/old_name"));

    let cloned = event.clone();
    assert_eq!(cloned.added, event.added);
    assert_eq!(cloned.modified, event.modified);
    assert_eq!(cloned.deleted, event.deleted);
    assert_eq!(cloned.moved, event.moved);
}

// --- deduplicate_and_merge_raw_notifier_events ---
// This function is non-pub (crate-private), so we test it through the module.

use instant::Instant;
use notify_debouncer_full::notify::event::{CreateKind, ModifyKind, RemoveKind, RenameMode};
use notify_debouncer_full::notify::{Event, EventKind};
use notify_debouncer_full::DebouncedEvent;

use crate::deduplicate_and_merge_raw_notifier_events;

fn make_event(kind: EventKind, paths: Vec<PathBuf>) -> DebouncedEvent {
    DebouncedEvent {
        event: Event {
            kind,
            paths,
            attrs: Default::default(),
        },
        time: Instant::now(),
    }
}

#[test]
fn dedup_create_event() {
    let events = vec![make_event(
        EventKind::Create(CreateKind::File),
        vec![PathBuf::from("/tmp/new_file")],
    )];
    let result = deduplicate_and_merge_raw_notifier_events(&events).unwrap();
    assert!(result.added.contains(&PathBuf::from("/tmp/new_file")));
    assert!(result.modified.is_empty());
    assert!(result.deleted.is_empty());
}

#[test]
fn dedup_modify_data_event() {
    let events = vec![make_event(
        EventKind::Modify(ModifyKind::Data(
            notify_debouncer_full::notify::event::DataChange::Content,
        )),
        vec![PathBuf::from("/tmp/existing_file")],
    )];
    let result = deduplicate_and_merge_raw_notifier_events(&events).unwrap();
    assert!(result
        .modified
        .contains(&PathBuf::from("/tmp/existing_file")));
    assert!(result.added.is_empty());
}

#[test]
fn dedup_create_then_remove_cancels_out() {
    let events = vec![
        make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/tmp/file")],
        ),
        make_event(
            EventKind::Remove(RemoveKind::File),
            vec![PathBuf::from("/tmp/file")],
        ),
    ];
    let result = deduplicate_and_merge_raw_notifier_events(&events);
    // Should produce empty event → error
    assert!(result.is_err());
}

#[test]
fn dedup_modify_then_remove_keeps_delete() {
    let events = vec![
        make_event(
            EventKind::Modify(ModifyKind::Data(
                notify_debouncer_full::notify::event::DataChange::Content,
            )),
            vec![PathBuf::from("/tmp/file")],
        ),
        make_event(
            EventKind::Remove(RemoveKind::File),
            vec![PathBuf::from("/tmp/file")],
        ),
    ];
    let result = deduplicate_and_merge_raw_notifier_events(&events).unwrap();
    assert!(result.deleted.contains(&PathBuf::from("/tmp/file")));
    assert!(result.modified.is_empty());
}

#[test]
fn dedup_create_then_modify_counts_as_added() {
    let events = vec![
        make_event(
            EventKind::Create(CreateKind::File),
            vec![PathBuf::from("/tmp/file")],
        ),
        make_event(
            EventKind::Modify(ModifyKind::Data(
                notify_debouncer_full::notify::event::DataChange::Content,
            )),
            vec![PathBuf::from("/tmp/file")],
        ),
    ];
    let result = deduplicate_and_merge_raw_notifier_events(&events).unwrap();
    assert!(result.added.contains(&PathBuf::from("/tmp/file")));
    assert!(!result.modified.contains(&PathBuf::from("/tmp/file")));
}

#[test]
fn dedup_rename_both_mode() {
    let events = vec![make_event(
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
        vec![PathBuf::from("/tmp/old"), PathBuf::from("/tmp/new")],
    )];
    let result = deduplicate_and_merge_raw_notifier_events(&events).unwrap();
    assert_eq!(
        result.moved.get(&PathBuf::from("/tmp/new")),
        Some(&PathBuf::from("/tmp/old"))
    );
}

#[test]
fn dedup_rename_from_then_to() {
    let events = vec![
        make_event(
            EventKind::Modify(ModifyKind::Name(RenameMode::From)),
            vec![PathBuf::from("/tmp/old")],
        ),
        make_event(
            EventKind::Modify(ModifyKind::Name(RenameMode::To)),
            vec![PathBuf::from("/tmp/new")],
        ),
    ];
    let result = deduplicate_and_merge_raw_notifier_events(&events).unwrap();
    assert_eq!(
        result.moved.get(&PathBuf::from("/tmp/new")),
        Some(&PathBuf::from("/tmp/old"))
    );
}

#[test]
fn dedup_empty_events_returns_error() {
    let events = vec![];
    assert!(deduplicate_and_merge_raw_notifier_events(&events).is_err());
}

#[test]
fn dedup_chained_rename_squashes() {
    // A→B, then B→C should result in A→C
    let events = vec![
        make_event(
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
            vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")],
        ),
        make_event(
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
            vec![PathBuf::from("/tmp/b"), PathBuf::from("/tmp/c")],
        ),
    ];
    let result = deduplicate_and_merge_raw_notifier_events(&events).unwrap();
    assert_eq!(
        result.moved.get(&PathBuf::from("/tmp/c")),
        Some(&PathBuf::from("/tmp/a"))
    );
    assert!(!result.moved.contains_key(&PathBuf::from("/tmp/b")));
}
