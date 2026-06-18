use crate::cloud_object::ObjectIdType;
use crate::ids::{
    ClientId, FolderId, GenericStringObjectId, HashableId, ServerId, ServerIdAndType, SyncId,
    parse_sqlite_id_to_uid,
};

// --- ServerId ---

#[test]
fn server_id_try_from_valid_22_char_string() {
    let id_str = "abcdefghijklmnopqrstuv";
    let id = ServerId::try_from(id_str);
    assert!(id.is_ok());
    assert_eq!(id.unwrap().to_string(), id_str);
}

#[test]
fn server_id_try_from_rejects_short_string() {
    let id = ServerId::try_from("short");
    assert!(id.is_err());
}

#[test]
fn server_id_try_from_rejects_long_string() {
    let id = ServerId::try_from("this_string_is_way_too_long_for_server_id");
    assert!(id.is_err());
}

#[test]
fn server_id_display_matches_input() {
    let id_str = "1234567890123456789012";
    let id = ServerId::try_from(id_str).unwrap();
    assert_eq!(format!("{id}"), id_str);
}

#[test]
fn server_id_debug_format() {
    let id_str = "abcdefghijklmnopqrstuv";
    let id = ServerId::try_from(id_str).unwrap();
    assert_eq!(format!("{id:?}"), format!("ServerId({id_str})"));
}

#[test]
#[should_panic(expected = "ServerId must be exactly 22 characters")]
fn server_id_from_string_lossy_panics_in_debug_for_short_input() {
    let _ = ServerId::from_string_lossy("abc");
}

#[test]
fn server_id_roundtrip_to_string_and_back() {
    let id_str = "ABCDEFGHIJKLMNOPQRSTUV";
    let id = ServerId::try_from(id_str).unwrap();
    let as_string: String = id.into();
    assert_eq!(as_string, id_str);
}

#[test]
fn server_id_from_i64_produces_22_chars() {
    let id = ServerId::from(123i64);
    assert_eq!(id.to_string().len(), 22);
    assert!(id.to_string().starts_with("test_uid"));
}

#[test]
fn server_id_from_i64_deterministic() {
    assert_eq!(ServerId::from(42i64), ServerId::from(42i64));
}

#[test]
fn server_id_from_i64_different_values_differ() {
    assert_ne!(ServerId::from(1i64), ServerId::from(2i64));
}

#[test]
fn server_id_serde_roundtrip() {
    let id_str = "abcdefghijklmnopqrstuv";
    let id = ServerId::try_from(id_str).unwrap();
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, format!("\"{id_str}\""));
    let deserialized: ServerId = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, id);
}

#[test]
fn server_id_sqlite_type_and_uid_hash() {
    let id = ServerId::from(100i64);
    let hash = id.sqlite_type_and_uid_hash(ObjectIdType::Notebook);
    assert!(hash.starts_with("Notebook-"));
}

// --- ClientId ---

#[test]
fn client_id_display_has_prefix() {
    let id = ClientId::new();
    let display = id.to_string();
    assert!(display.starts_with("Client-"));
}

#[test]
fn client_id_hash_roundtrip() {
    let id = ClientId::new();
    let hash = id.to_hash();
    let recovered = ClientId::from_hash(&hash);
    assert!(recovered.is_some());
    assert_eq!(recovered.unwrap(), id);
}

#[test]
fn client_id_from_hash_fails_without_prefix() {
    assert!(ClientId::from_hash("not-a-client-id").is_none());
}

#[test]
fn client_id_from_hash_fails_with_invalid_uuid() {
    assert!(ClientId::from_hash("Client-not-a-uuid").is_none());
}

#[test]
fn client_id_sqlite_hash_starts_with_client() {
    let id = ClientId::new();
    assert!(id.sqlite_hash().starts_with("Client-"));
}

// --- SyncId ---

#[test]
fn sync_id_from_server_id() {
    let server_id = ServerId::from(1i64);
    let sync_id = SyncId::ServerId(server_id);
    assert!(sync_id.into_server().is_some());
}

#[test]
fn sync_id_from_client_id() {
    let client_id = ClientId::new();
    let sync_id = SyncId::ClientId(client_id);
    assert!(sync_id.into_client().is_some());
}

#[test]
fn sync_id_into_server_returns_none_for_client() {
    let sync_id = SyncId::ClientId(ClientId::new());
    assert!(sync_id.into_server().is_none());
}

#[test]
fn sync_id_into_client_returns_none_for_server() {
    let sync_id = SyncId::ServerId(ServerId::from(1i64));
    assert!(sync_id.into_client().is_none());
}

#[test]
fn sync_id_uid_for_server() {
    let server_id = ServerId::from(1i64);
    let sync_id = SyncId::ServerId(server_id);
    assert!(!sync_id.uid().is_empty());
}

#[test]
fn sync_id_uid_for_client() {
    let client_id = ClientId::new();
    let sync_id = SyncId::ClientId(client_id);
    assert!(sync_id.uid().starts_with("Client-"));
}

#[test]
fn sync_id_serde_roundtrip_server() {
    let server_id = ServerId::from(42i64);
    let sync_id = SyncId::ServerId(server_id);
    let json = serde_json::to_string(&sync_id).unwrap();
    let deserialized: SyncId = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.uid(), sync_id.uid());
}

#[test]
fn sync_id_serde_roundtrip_client() {
    let client_id = ClientId::new();
    let sync_id = SyncId::ClientId(client_id);
    let json = serde_json::to_string(&sync_id).unwrap();
    let deserialized: SyncId = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.uid(), sync_id.uid());
}

#[test]
fn sync_id_display_for_server() {
    let server_id = ServerId::from(1i64);
    let sync_id = SyncId::ServerId(server_id);
    assert_eq!(sync_id.to_string(), server_id.to_string());
}

#[test]
fn sync_id_display_for_client() {
    let client_id = ClientId::new();
    let display = client_id.to_string();
    let sync_id = SyncId::ClientId(client_id);
    assert_eq!(sync_id.to_string(), display);
}

#[test]
fn sync_id_from_server_id_impl() {
    let server_id = ServerId::from(1i64);
    let sync_id: SyncId = server_id.into();
    assert!(matches!(sync_id, SyncId::ServerId(_)));
}

// --- ObjectIdType ---

#[test]
fn object_id_type_sqlite_prefix_notebook() {
    assert_eq!(ObjectIdType::Notebook.sqlite_prefix(), "Notebook");
}

#[test]
fn object_id_type_sqlite_prefix_workflow() {
    assert_eq!(ObjectIdType::Workflow.sqlite_prefix(), "Workflow");
}

#[test]
fn object_id_type_sqlite_prefix_folder() {
    assert_eq!(ObjectIdType::Folder.sqlite_prefix(), "Folder");
}

#[test]
fn object_id_type_sqlite_prefix_generic_string_object() {
    assert_eq!(
        ObjectIdType::GenericStringObject.sqlite_prefix(),
        "GenericStringObject"
    );
}

// --- FolderId ---

#[test]
fn folder_id_hashable_roundtrip() {
    let folder_id = FolderId::from(1i64);
    let hash = folder_id.to_hash();
    assert!(hash.starts_with("Folder-"));
    let recovered = FolderId::from_hash(&hash);
    assert!(recovered.is_some());
}

#[test]
fn folder_id_into_sync_id() {
    let folder_id = FolderId::from(1i64);
    let sync_id: SyncId = folder_id.into();
    assert!(matches!(sync_id, SyncId::ServerId(_)));
}

// --- GenericStringObjectId ---

#[test]
fn generic_string_object_id_hashable_roundtrip() {
    let id = GenericStringObjectId::from(1i64);
    let hash = id.to_hash();
    assert!(hash.starts_with("GenericStringObject-"));
    let recovered = GenericStringObjectId::from_hash(&hash);
    assert!(recovered.is_some());
}

#[test]
fn generic_string_object_id_uid() {
    let id = GenericStringObjectId::from(1i64);
    assert!(!id.uid().is_empty());
}

// --- parse_sqlite_id_to_uid ---

#[test]
fn parse_sqlite_id_to_uid_extracts_uid() {
    let result = parse_sqlite_id_to_uid("Notebook-abc123".to_string());
    assert_eq!(result.unwrap(), "abc123");
}

#[test]
fn parse_sqlite_id_to_uid_handles_multiple_dashes() {
    let result = parse_sqlite_id_to_uid("Notebook-some-complex-uid".to_string());
    assert_eq!(result.unwrap(), "uid");
}

// --- ServerIdAndType ---

#[test]
fn server_id_and_type_sqlite_hash() {
    let id_and_type = ServerIdAndType {
        id: ServerId::from(1i64),
        id_type: ObjectIdType::Workflow,
    };
    let hash = id_and_type.sqlite_type_and_uid_hash();
    assert!(hash.starts_with("Workflow-"));
}
