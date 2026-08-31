use super::*;

#[test]
fn tool_call_reports_missing_values_as_tool_errors() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model(|_| Ok::<ObjectResponse, String>(object_response(json!({}))))
        .expect("tool should register");

    let result = server.call_tool(&PlainRequest::descriptor().tool_name(), Some(json!({})));

    assert_eq!(result.is_error, Some(true));
    assert!(
        result.content[0]
            .as_text()
            .is_some_and(|text| text.text.contains("missing required field `title`"))
    );
}

#[test]
fn koruma_validation_runs_before_submit_handler() {
    let mut server = test_server();
    form::<ValidatedRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({ "name": request.name })))
        })
        .expect("tool should register");

    let result = server.call_tool(
        &ValidatedRequest::descriptor().tool_name(),
        Some(json!({ "name": "" })),
    );

    assert_eq!(result.is_error, Some(true));
    let error = result
        .structured_content
        .as_ref()
        .expect("validation should be structured")
        .get("error")
        .expect("structured validation error");
    assert_eq!(error["kind"], json!("validation"));
    let detail = error["detail"]
        .as_str()
        .expect("validation detail should be text");
    assert!(!detail.is_empty());
    let details = error["details"]
        .as_array()
        .expect("validation details should be an array");
    assert_eq!(details.len(), 1);
    assert_eq!(details[0]["scope"], json!("field"));
    assert_eq!(details[0]["field"], json!("name"));
    assert_eq!(details[0]["validator"], json!("NonEmptyValidation"));
    assert_eq!(details[0]["path"], json!("NonEmptyValidation"));
    assert_eq!(details[0]["target"], json!("default"));
    assert_eq!(details[0]["message"], json!(detail));
    assert!(
        result.content[0]
            .as_text()
            .is_some_and(|text| text.text.contains("validation failed"))
    );
}

#[test]
fn koruma_newtype_conversion_reports_structured_validation_before_submit_handler() {
    let mut server = test_server();
    let called = Arc::new(Mutex::new(false));
    let called_by_handler = Arc::clone(&called);
    form::<NewtypeConvertedRequest>(&mut server)
        .model(move |request| {
            *called_by_handler.lock().expect("handler flag lock") = true;
            Ok::<ObjectResponse, String>(object_response(json!({
                "code": request.code.into_inner()
            })))
        })
        .expect("tool should register");

    let result = server.call_tool(
        &NewtypeConvertedRequest::descriptor().tool_name(),
        Some(json!({ "code": "x" })),
    );

    assert_eq!(result.is_error, Some(true));
    assert!(
        !*called.lock().expect("handler flag lock"),
        "invalid newtype input should fail before handler execution"
    );
    let error = result
        .structured_content
        .as_ref()
        .expect("validation should be structured")
        .get("error")
        .expect("structured validation error");
    assert_eq!(error["kind"], json!("validation"));
    let details = error["details"]
        .as_array()
        .expect("validation details should be an array");
    assert_eq!(details.len(), 1);
    assert_eq!(details[0]["scope"], json!("field"));
    assert_eq!(details[0]["field"], json!("code"));
    assert_eq!(details[0]["validator"], json!("conversion"));
    assert!(
        details[0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Invalid value for field 'code'"))
    );
}
