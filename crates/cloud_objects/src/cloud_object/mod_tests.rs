use std::str::FromStr;

use crate::cloud_object::{
    GENERIC_STRING_OBJECT_PREFIX, GenericStringObjectFormat, JSON_OBJECT_PREFIX, JsonObjectType,
    ObjectIdType, ObjectType,
};

// --- ObjectIdType::sqlite_prefix ---

#[test]
fn object_id_type_sqlite_prefix_values() {
    assert_eq!(ObjectIdType::Notebook.sqlite_prefix(), "Notebook");
    assert_eq!(ObjectIdType::Workflow.sqlite_prefix(), "Workflow");
    assert_eq!(ObjectIdType::Folder.sqlite_prefix(), "Folder");
    assert_eq!(
        ObjectIdType::GenericStringObject.sqlite_prefix(),
        "GenericStringObject"
    );
}

// --- ObjectType::from_str ---

#[test]
fn object_type_from_str_notebook() {
    let obj = ObjectType::from_str("notebook").unwrap();
    assert!(matches!(obj, ObjectType::Notebook));
}

#[test]
fn object_type_from_str_workflow() {
    let obj = ObjectType::from_str("workflow").unwrap();
    assert!(matches!(obj, ObjectType::Workflow));
}

#[test]
fn object_type_from_str_prompt_maps_to_workflow() {
    let obj = ObjectType::from_str("prompt").unwrap();
    assert!(matches!(obj, ObjectType::Workflow));
}

#[test]
fn object_type_from_str_folder() {
    let obj = ObjectType::from_str("folder").unwrap();
    assert!(matches!(obj, ObjectType::Folder));
}

#[test]
fn object_type_from_str_env_vars() {
    let obj = ObjectType::from_str("env-vars").unwrap();
    assert!(matches!(
        obj,
        ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
            JsonObjectType::EnvVarCollection
        ))
    ));
}

#[test]
fn object_type_from_str_unknown_fails() {
    assert!(ObjectType::from_str("unknown_type").is_err());
}

// --- ObjectType::Display ---

#[test]
fn object_type_display_notebook() {
    assert_eq!(ObjectType::Notebook.to_string(), "notebook");
}

#[test]
fn object_type_display_workflow() {
    assert_eq!(ObjectType::Workflow.to_string(), "workflow");
}

#[test]
fn object_type_display_folder() {
    assert_eq!(ObjectType::Folder.to_string(), "folder");
}

#[test]
fn object_type_display_env_var_collection() {
    let obj = ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
        JsonObjectType::EnvVarCollection,
    ));
    assert_eq!(obj.to_string(), "env-vars");
}

// --- ObjectType → ObjectIdType ---

#[test]
fn object_type_into_object_id_type() {
    assert_eq!(
        ObjectIdType::from(ObjectType::Notebook),
        ObjectIdType::Notebook
    );
    assert_eq!(
        ObjectIdType::from(ObjectType::Workflow),
        ObjectIdType::Workflow
    );
    assert_eq!(ObjectIdType::from(ObjectType::Folder), ObjectIdType::Folder);
    assert_eq!(
        ObjectIdType::from(ObjectType::GenericStringObject(
            GenericStringObjectFormat::Json(JsonObjectType::EnvVarCollection)
        )),
        ObjectIdType::GenericStringObject
    );
}

// --- ObjectType::sqlite_object_type_as_str ---

#[test]
fn sqlite_object_type_notebook() {
    assert_eq!(ObjectType::Notebook.sqlite_object_type_as_str(), "NOTEBOOK");
}

#[test]
fn sqlite_object_type_workflow() {
    assert_eq!(ObjectType::Workflow.sqlite_object_type_as_str(), "WORKFLOW");
}

#[test]
fn sqlite_object_type_folder() {
    assert_eq!(ObjectType::Folder.sqlite_object_type_as_str(), "FOLDER");
}

#[test]
fn sqlite_object_type_generic_string_object() {
    let obj = ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
        JsonObjectType::EnvVarCollection,
    ));
    let s = obj.sqlite_object_type_as_str();
    assert!(s.starts_with(GENERIC_STRING_OBJECT_PREFIX));
    assert!(s.contains(JSON_OBJECT_PREFIX));
}

// --- JsonObjectType ---

#[test]
fn json_object_type_as_str_roundtrip() {
    let variants = [
        JsonObjectType::Preference,
        JsonObjectType::EnvVarCollection,
        JsonObjectType::WorkflowEnum,
        JsonObjectType::AIFact,
        JsonObjectType::MCPServer,
        JsonObjectType::AIExecutionProfile,
        JsonObjectType::TemplatableMCPServer,
        JsonObjectType::CloudEnvironment,
        JsonObjectType::ScheduledAmbientAgent,
        JsonObjectType::CloudAgentConfig,
    ];

    for variant in &variants {
        let s = variant.as_str();
        let recovered = JsonObjectType::try_from(s).unwrap();
        assert_eq!(*variant, recovered, "roundtrip failed for {s}");
    }
}

#[test]
fn json_object_type_try_from_unknown_fails() {
    assert!(JsonObjectType::try_from("UNKNOWN").is_err());
}

// --- GenericStringObjectFormat ---

#[test]
fn generic_string_object_format_to_string() {
    let fmt = GenericStringObjectFormat::Json(JsonObjectType::Preference);
    let s = fmt.to_string();
    assert!(s.starts_with(GENERIC_STRING_OBJECT_PREFIX));
    assert!(s.contains(JSON_OBJECT_PREFIX));
    assert!(s.ends_with("PREFERENCE"));
}
