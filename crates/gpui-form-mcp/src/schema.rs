use serde_json::Value;

use crate::*;

pub(crate) fn input_schema_for_fields(fields: &[McpField]) -> McpSchema {
    input_schema_for_fields_with_required(fields, true)
}

pub(crate) fn optional_input_schema_for_fields(fields: &[McpField]) -> McpSchema {
    input_schema_for_fields_with_required(fields, false)
}

pub(crate) fn input_schema_for_fields_with_required(
    fields: &[McpField],
    include_required: bool,
) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    let mut required = Vec::new();

    for field in fields {
        properties.insert(field.name().to_string(), schema_for_field(*field));
        if include_required && field.required() {
            required.push(field.name());
        }
    }

    component_shape_mcp::object_schema(properties, required)
}

pub(crate) fn open_editor_input_schema(fields: &[McpField]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "values".to_string(),
        optional_input_schema_for_fields(fields),
    );
    component_shape_mcp::object_schema(properties, std::iter::empty::<&str>())
}

pub(crate) fn patch_editor_input_schema(fields: &[McpField]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("session_id".to_string(), McpSchema::string());
    properties.insert(
        "values".to_string(),
        optional_input_schema_for_fields(fields),
    );
    properties.insert("expected_revision".to_string(), expected_revision_schema());
    properties.insert("clear".to_string(), field_name_array_schema(fields));
    properties.insert(
        "replace".to_string(),
        McpSchema::boolean()
            .with_default(false)
            .with_description(
                "When true, replace the session values with this patch instead of merging into the existing values.",
            ),
    );
    component_shape_mcp::object_schema(properties, ["session_id"])
}

pub(crate) fn edit_session_snapshot_output_schema(fields: &[McpField]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("form".to_string(), McpSchema::string());
    properties.insert("form_name".to_string(), McpSchema::string());
    properties.insert("session_id".to_string(), McpSchema::string());
    properties.insert(
        "revision".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "fields".to_string(),
        edit_session_fields_output_schema(fields),
    );
    properties.insert(
        "present_fields".to_string(),
        field_name_array_schema(fields),
    );
    properties.insert(
        "missing_required".to_string(),
        field_name_array_schema(fields),
    );
    properties.insert("valid".to_string(), McpSchema::boolean());
    properties.insert(
        "errors".to_string(),
        component_shape_mcp::array_schema(validation_issue_schema()),
    );
    properties.insert(
        "values".to_string(),
        optional_input_schema_for_fields(fields),
    );
    properties.insert(
        "submit_arguments".to_string(),
        optional_input_schema_for_fields(fields),
    );
    properties.insert("cleanup".to_string(), edit_session_cleanup_schema());

    component_shape_mcp::object_schema(
        properties,
        [
            "form",
            "form_name",
            "session_id",
            "revision",
            "fields",
            "present_fields",
            "missing_required",
            "valid",
            "errors",
            "values",
            "submit_arguments",
        ],
    )
}

pub(crate) fn edit_session_list_output_schema(fields: &[McpField]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("form".to_string(), McpSchema::string());
    properties.insert("form_name".to_string(), McpSchema::string());
    properties.insert("session_limit".to_string(), session_limit_schema());
    properties.insert(
        "session_idle_timeout_ms".to_string(),
        session_idle_timeout_schema(),
    );
    properties.insert(
        "session_count".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "sessions".to_string(),
        component_shape_mcp::array_schema(edit_session_snapshot_output_schema(fields)),
    );
    properties.insert("cleanup".to_string(), edit_session_cleanup_schema());

    component_shape_mcp::object_schema(
        properties,
        [
            "form",
            "form_name",
            "session_limit",
            "session_idle_timeout_ms",
            "session_count",
            "sessions",
            "cleanup",
        ],
    )
}

pub(crate) fn edit_session_cleanup_schema() -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "expired_count".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "expired_session_ids".to_string(),
        component_shape_mcp::array_schema(McpSchema::string()),
    );
    properties.insert(
        "evicted_count".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "evicted_session_ids".to_string(),
        component_shape_mcp::array_schema(McpSchema::string()),
    );

    component_shape_mcp::object_schema(
        properties,
        [
            "expired_count",
            "expired_session_ids",
            "evicted_count",
            "evicted_session_ids",
        ],
    )
}

pub(crate) fn close_editor_output_schema() -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("session_id".to_string(), McpSchema::string());
    properties.insert("closed".to_string(), McpSchema::boolean());

    component_shape_mcp::object_schema(properties, ["session_id", "closed"])
}

pub(crate) fn close_all_editor_output_schema(descriptor: McpFormDescriptor) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "form".to_string(),
        McpSchema::string().with_const(descriptor.tool_name()),
    );
    properties.insert(
        "form_name".to_string(),
        McpSchema::string().with_const(descriptor.form_name()),
    );
    properties.insert(
        "closed_count".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "session_ids".to_string(),
        component_shape_mcp::array_schema(McpSchema::string()),
    );

    component_shape_mcp::object_schema(
        properties,
        ["form", "form_name", "closed_count", "session_ids"],
    )
}

pub(crate) fn edit_session_fields_output_schema(fields: &[McpField]) -> McpSchema {
    if fields.is_empty() {
        return component_shape_mcp::array_schema(component_shape_mcp::object_schema(
            McpSchemaProperties::new(),
            std::iter::empty::<&str>(),
        ));
    }

    component_shape_mcp::array_schema(McpSchema::one_of(
        fields
            .iter()
            .map(|field| edit_session_field_output_schema(*field)),
    ))
}

pub(crate) fn edit_session_field_output_schema(field: McpField) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "name".to_string(),
        McpSchema::string().with_enum_values([field.name()]),
    );
    properties.insert(
        "value_type".to_string(),
        McpSchema::string().with_enum_values([field.value_type().as_str()]),
    );
    properties.insert(
        "label".to_string(),
        McpSchema::string().with_const(field.label()),
    );
    properties.insert(
        "description".to_string(),
        component_shape_mcp::nullable_schema(McpSchema::string()),
    );
    properties.insert(
        "examples".to_string(),
        component_shape_mcp::array_schema(McpSchema::string()),
    );
    properties.insert(
        "required".to_string(),
        McpSchema::boolean().with_const(field.required()),
    );
    properties.insert("present".to_string(), McpSchema::boolean());
    properties.insert("has_value".to_string(), McpSchema::boolean());
    properties.insert(
        "value".to_string(),
        component_shape_mcp::nullable_schema(schema_for_field(field)),
    );
    properties.insert("missing".to_string(), McpSchema::boolean());
    properties.insert(
        "errors".to_string(),
        component_shape_mcp::array_schema(validation_issue_schema()),
    );
    properties.insert(
        "has_default".to_string(),
        McpSchema::boolean().with_const(field.has_default()),
    );
    properties.insert(
        "component_backed".to_string(),
        McpSchema::boolean().with_const(field.component_backed()),
    );
    properties.insert(
        "schema".to_string(),
        McpSchema::object().with_const(schema_for_field(field).into_value()),
    );

    component_shape_mcp::object_schema(
        properties,
        [
            "name",
            "label",
            "description",
            "examples",
            "value_type",
            "required",
            "present",
            "has_value",
            "value",
            "missing",
            "errors",
            "has_default",
            "component_backed",
            "schema",
        ],
    )
}

pub(crate) fn field_name_array_schema(fields: &[McpField]) -> McpSchema {
    McpSchema::array(McpSchema::string().with_enum_values(fields.iter().map(|field| field.name())))
        .with_unique_items(true)
}

pub(crate) fn validation_issue_schema() -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "scope".to_string(),
        McpSchema::string().with_enum_values(["form", "field", "element"]),
    );
    properties.insert("message".to_string(), McpSchema::string());
    properties.insert("field".to_string(), McpSchema::string());
    properties.insert("validator".to_string(), McpSchema::string());
    properties.insert("path".to_string(), McpSchema::string());
    properties.insert("label".to_string(), McpSchema::string());
    properties.insert(
        "target".to_string(),
        McpSchema::string().with_enum_values(["default", "full", "unwrapped"]),
    );
    properties.insert(
        "element_index".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "params".to_string(),
        component_shape_mcp::array_schema(validation_param_schema()),
    );

    component_shape_mcp::object_schema(properties, ["scope", "message"])
}

pub(crate) fn validation_param_schema() -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("name".to_string(), McpSchema::string());
    properties.insert("value".to_string(), McpSchema::string());
    properties.insert("expr".to_string(), McpSchema::string());

    component_shape_mcp::object_schema(properties, ["name"])
}

pub(crate) fn expected_revision_schema() -> McpSchema {
    McpSchema::integer().with_minimum(0_u64).with_description(
        "When set, apply this operation only if the edit session is still at this revision.",
    )
}

pub(crate) fn session_limit_schema() -> McpSchema {
    component_shape_mcp::nullable_schema(McpSchema::integer().with_minimum(1_u64).with_description(
        "Maximum active edit sessions retained for this form, or null when unlimited.",
    ))
}

pub(crate) fn session_idle_timeout_schema() -> McpSchema {
    component_shape_mcp::nullable_schema(McpSchema::integer().with_minimum(0_u64).with_description(
        "Idle milliseconds after which a session is expired, or null when idle expiry is disabled.",
    ))
}

pub(crate) fn schema_for_field(field: McpField) -> McpSchema {
    let base = (field.tool_value_schema())();
    let mut schema = if field.presence().optional() {
        component_shape_mcp::nullable_schema(base)
    } else {
        base
    };

    if let Some(object) = schema.as_object_mut() {
        object.insert(
            "x-rustType".to_string(),
            Value::String(field.value_type().as_str().to_string()),
        );
        object.insert(
            "x-gpuiFormPresence".to_string(),
            Value::String(format!("{:?}", field.presence())),
        );
        object.insert("title".to_string(), Value::String(field.label()));
        object.insert("x-gpuiFormLabel".to_string(), Value::String(field.label()));
        if let Some(label) = field.explicit_label() {
            object.insert(
                "x-gpuiFormExplicitLabel".to_string(),
                Value::String(label.to_string()),
            );
        }
        if let Some(description) = field.description() {
            object.insert(
                "description".to_string(),
                Value::String(description.to_string()),
            );
            object.insert(
                "x-gpuiFormDescription".to_string(),
                Value::String(description.to_string()),
            );
        }
        if !field.examples().is_empty() {
            object.insert(
                "examples".to_string(),
                Value::Array(
                    field
                        .examples()
                        .iter()
                        .map(|example| Value::String((*example).to_string()))
                        .collect(),
                ),
            );
        }
        if field.component_backed() {
            object.insert("x-gpuiFormComponentBacked".to_string(), Value::Bool(true));
        }
        if field.mcp_input().supported() {
            object.insert(
                "x-gpuiFormComponentMcpInput".to_string(),
                component_shape_mcp::schema_for_input(field.mcp_input()).into_value(),
            );
        }
        if field.has_default() {
            object.insert("x-gpuiFormHasDefault".to_string(), Value::Bool(true));
        }
        component_shape_mcp::apply_validation_schema_metadata(
            object,
            "x-gpuiFormValidation",
            field.validation_rules(),
        );
    }

    schema
}
