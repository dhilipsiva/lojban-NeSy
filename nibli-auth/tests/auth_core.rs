//! Core authorization scenarios (A1).

use nibli_auth::{Authorizer, BUILTIN_POLICY, POLICY_VERSION, Verdict};
use nibli_session::CoreSession;

fn loaded() -> Authorizer {
    let mut a = Authorizer::new();
    a.load_policy(None).expect("load_policy");
    a
}

#[test]
fn builtin_policy_compiles() {
    let s = CoreSession::new();
    s.assert_text(BUILTIN_POLICY)
        .expect("builtin policy must assert");
}

#[test]
fn policy_version_constant() {
    let a = loaded();
    assert_eq!(a.policy_version(), POLICY_VERSION);
    assert!(a.policy_loaded());
}

#[test]
fn owner_can_update() {
    let mut a = loaded();
    a.assert_facts("owns(Alice, Doc1).").expect("owns fact");
    let d = a.can("Alice", "update", "Doc1", "").expect("can");
    assert!(d.allowed, "owner should be allowed: {d:?}");
    assert_eq!(d.verdict, Verdict::True);
}

#[test]
fn stranger_denied() {
    let mut a = loaded();
    a.assert_facts("owns(Alice, Doc1).").expect("owns fact");
    let d = a.can("Bob", "update", "Doc1", "").expect("can");
    assert!(!d.allowed, "stranger must not be allowed: {d:?}");
}

#[test]
fn admin_can_update_any() {
    let mut a = loaded();
    // Admin rules require resource($r) in the body so $r is constrained.
    a.assert_facts("has_role(Bob, \"admin\").\nresource(Doc1).")
        .expect("role+resource");
    let d = a.can("Bob", "update", "Doc1", "").expect("can");
    assert!(d.allowed, "admin should update any: {d:?}");
}

#[test]
fn owner_visible_attrs_via_candidates() {
    let mut a = loaded();
    a.assert_facts("owns(Alice, Doc1).").expect("owns");
    let fields = a
        .allowed_fields("Alice", "read", "Doc1", "", &["title", "body", "secret"])
        .expect("fields");
    assert!(
        fields.contains(&"title".to_string()) && fields.contains(&"body".to_string()),
        "owner should see candidate attrs: {fields:?}"
    );
}

#[test]
fn stranger_no_visible_attrs() {
    let mut a = loaded();
    a.assert_facts("owns(Alice, Doc1).").expect("owns");
    let fields = a
        .allowed_fields("Bob", "read", "Doc1", "", &["title", "body"])
        .expect("fields");
    assert!(fields.is_empty(), "stranger fields: {fields:?}");
}

#[test]
fn explain_returns_proof_json_when_true() {
    let mut a = loaded();
    a.assert_facts("owns(Alice, Doc1).").expect("owns");
    let ex = a.explain("Alice", "read", "Doc1", "").expect("explain");
    assert!(ex.decision.allowed);
    assert!(
        ex.proof_json.as_ref().is_some_and(|j| !j.is_empty()),
        "expected non-empty proof JSON"
    );
}

#[test]
fn context_is_ephemeral() {
    let mut a = loaded();
    // No durable owns. Context grants Alice ownership for one call.
    let d = a
        .can("Alice", "update", "Doc1", "owns(Alice, Doc1).")
        .expect("can with context");
    assert!(d.allowed, "context owns should allow: {d:?}");
    // After retract, same call without context must deny.
    let d2 = a.can("Alice", "update", "Doc1", "").expect("can bare");
    assert!(!d2.allowed, "ephemeral owns must not persist: {d2:?}");
}

#[test]
fn same_tenant_read() {
    let mut a = loaded();
    a.assert_facts(
        r#"
in_tenant(Alice, "acme").
resource_tenant(Doc1, "acme").
"#,
    )
    .expect("tenant facts");
    let d = a.can("Alice", "read", "Doc1", "").expect("can");
    assert!(d.allowed, "same tenant read: {d:?}");
    let d2 = a.can("Alice", "update", "Doc1", "").expect("can update");
    assert!(!d2.allowed, "tenant rule is read-only: {d2:?}");
}

#[test]
fn can_requires_load_policy() {
    let mut a = Authorizer::new();
    let err = a.can("Alice", "read", "Doc1", "").unwrap_err();
    assert!(
        err.message.contains("policy not loaded"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn decision_cache_hit_stable() {
    let mut a = loaded();
    a.assert_facts("owns(Alice, Doc1).").expect("owns");
    let d1 = a.can("Alice", "read", "Doc1", "").expect("1");
    let d2 = a.can("Alice", "read", "Doc1", "").expect("2");
    assert_eq!(d1, d2);
}

#[test]
fn can_any_batch_authorization() {
    let mut a = loaded();
    let ctx = r#"
owns(Alice, Doc1).
in_tenant(Alice, "acme").
resource_tenant(Doc2, "acme").
resource_tenant(Doc3, "other").
"#;
    let candidates = vec!["Doc1", "Doc2", "Doc3", "Doc4"];
    let results = a
        .can_any("Alice", "read", &candidates, ctx)
        .expect("can_any");
    assert_eq!(
        results,
        vec![
            ("Doc1".to_string(), true),
            ("Doc2".to_string(), true),
            ("Doc3".to_string(), false),
            ("Doc4".to_string(), false),
        ]
    );

    let tls_results =
        nibli_auth::tls::can_any("Alice", "read", &candidates, ctx).expect("tls::can_any");
    assert_eq!(tls_results, results);
}

#[test]
fn grant_rule_action_specific() {
    let mut a = loaded();
    let ctx = r#"
grant(Alice, "edit", Doc1).
grant(Bob, "delete", Doc2).
"#;
    let d1 = a.can("Alice", "edit", "Doc1", ctx).expect("can edit");
    assert!(d1.allowed, "grant(Alice, edit, Doc1) should allow: {d1:?}");

    let d1_wrong = a.can("Alice", "delete", "Doc1", ctx).expect("can delete");
    assert!(
        !d1_wrong.allowed,
        "grant edit should not grant delete: {d1_wrong:?}"
    );

    let d2 = a.can("Bob", "delete", "Doc2", ctx).expect("can delete");
    assert!(d2.allowed, "grant(Bob, delete, Doc2) should allow: {d2:?}");
}

#[test]
fn tls_load_custom_policy() {
    nibli_auth::tls::reset_thread_auth();
    nibli_auth::tls::load_policy(Some(
        "all $a, $r: has_role($a, \"super\") & resource($r) -> authorized($a, \"all\", $r).",
    ))
    .unwrap();
    let ctx = "has_role(Alice, \"super\").\nresource(Doc100).";
    let d = nibli_auth::tls::can("Alice", "all", "Doc100", ctx).unwrap();
    assert!(d.allowed, "custom loaded policy should allow: {d:?}");
    nibli_auth::tls::reset_thread_auth();
}
