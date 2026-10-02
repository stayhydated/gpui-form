use super::session::EditSessions;
use crate::McpFormEditorOptions;
use proptest::prelude::*;
use serde_json::Map;
use std::{
    collections::{BTreeSet, VecDeque},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
enum Operation {
    Open(i16),
    Read(u8),
    Close(u8),
    CloseAll,
}

fn operation() -> impl Strategy<Value = Operation> {
    prop_oneof![
        4 => any::<i16>().prop_map(Operation::Open),
        2 => any::<u8>().prop_map(Operation::Read),
        2 => any::<u8>().prop_map(Operation::Close),
        1 => Just(Operation::CloseAll),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn bounded_session_lifecycle_matches_fifo_model(
        limit in 1_usize..9,
        operations in prop::collection::vec(operation(), 0..64),
    ) {
        let options = McpFormEditorOptions::default()
            .with_session_limit(limit).without_session_idle_timeout();
        let mut sessions = EditSessions::new(options);
        // A queue models insertion order independently of the production BTreeMap.
        let mut model = VecDeque::<(String, i16)>::new();
        let mut next_id = 1_u64;
        for operation in operations {
            match operation {
                Operation::Open(holder) => {
                    let opened = sessions.open(holder, BTreeSet::new(), Map::new());
                    let expected_id = next_id.to_string();
                    next_id += 1;
                    prop_assert_eq!(&opened.session_id, &expected_id);
                    prop_assert!(opened.cleanup.expired_session_ids.is_empty());
                    model.push_back((expected_id, holder));
                    let expected_evicted = if model.len() > limit {
                        vec![model.pop_front().unwrap().0]
                    } else {
                        Vec::new()
                    };
                    prop_assert_eq!(opened.cleanup.evicted_session_ids, expected_evicted);
                },
                Operation::Read(slot) => {
                    // One extra slot deliberately exercises a nonexistent ID;
                    // references remain meaningful when earlier operations shrink.
                    let index = usize::from(slot) % (model.len() + 1);
                    match model.get(index) {
                        Some((id, holder)) => prop_assert_eq!(sessions.get(id).unwrap().holder, *holder),
                        None => prop_assert_eq!(sessions.get("missing").unwrap_err(),
                            crate::McpToolError::invalid_field_value("session_id", "missing")),
                    }
                },
                Operation::Close(slot) => {
                    let index = usize::from(slot) % (model.len() + 1);
                    if let Some((id, _)) = model.remove(index) {
                        prop_assert_eq!(sessions.close(&id), Ok(true));
                    } else {
                        prop_assert_eq!(sessions.close("missing"), Err(
                            crate::McpToolError::invalid_field_value("session_id", "missing"),
                        ));
                    }
                },
                Operation::CloseAll => {
                    let mut expected = model.drain(..).map(|(id, _)| id).collect::<Vec<_>>();
                    expected.sort();
                    prop_assert_eq!(sessions.close_all(), expected);
                },
            }
            let actual = sessions.sessions.iter()
                .map(|(id, session)| (id.clone(), session.holder)).collect::<BTreeSet<_>>();
            let expected = model.iter().cloned().collect::<BTreeSet<_>>();
            prop_assert_eq!(actual, expected);
            prop_assert_eq!(sessions.next_id, next_id);
            prop_assert!(sessions.sessions.len() <= limit);
        }
    }

    #[test]
    fn expiration_uses_inclusive_age_boundary_without_sleeping(
        timeout_seconds in 0_u64..121,
        ages in prop::collection::vec(0_u64..121, 0..32),
    ) {
        let options = McpFormEditorOptions::default()
            .without_session_limit().without_session_idle_timeout();
        let mut sessions = EditSessions::new(options);
        let mut expected_expired = Vec::new();
        let mut expected_retained = BTreeSet::new();
        // Use one logical instant for every stored age and the expiry call.
        let now = Instant::now();
        for (holder, age) in ages.into_iter().enumerate() {
            let id = sessions.open(holder, BTreeSet::new(), Map::new()).session_id;
            if age >= timeout_seconds {
                expected_expired.push(id.clone());
            } else {
                expected_retained.insert((id.clone(), holder));
            }
            sessions.sessions.get_mut(&id).unwrap().last_accessed_at =
                now - Duration::from_secs(age);
        }
        sessions.options = options.with_session_idle_timeout(Duration::from_secs(timeout_seconds));
        expected_expired.sort();
        prop_assert_eq!(sessions.expire_idle_sessions(now), expected_expired);
        let retained = sessions.sessions.iter()
            .map(|(id, session)| (id.clone(), session.holder)).collect::<BTreeSet<_>>();
        prop_assert_eq!(retained, expected_retained);
        prop_assert!(sessions.expire_idle_sessions(now).is_empty());
    }
}
