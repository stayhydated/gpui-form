#[cfg(feature = "component-shape")]
use gpui_form_component::infinite_select::{
    InfiniteSelect as InfiniteSelectComponent, InfiniteSelectItem, InfiniteSelectOptions,
};
use gpui_form_component::infinite_select::{
    InfiniteSelectKeyPath, InfiniteSelectPath, InfiniteSelectPathErrorReason,
    InfiniteSelectPathSegment, InfiniteSelectValue as _, build_from_key_path, build_from_path,
};
use gpui_form_component_derive::InfiniteSelect;
#[cfg(feature = "component-shape")]
use gpui_kit as gpui;
#[cfg(feature = "component-shape")]
use gpui_kit::component::select::{SelectDelegate, SelectItem as _};

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
enum CaliforniaCity {
    #[default]
    LosAngeles,
    SanFrancisco,
}

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
enum TexasCity {
    #[default]
    Houston,
    Dallas,
    Austin,
}

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
enum NewYorkCity {
    #[default]
    NewYorkCity,
    Buffalo,
}

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
enum OntarioCity {
    #[default]
    Toronto,
    Ottawa,
}

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
enum QuebecCity {
    #[default]
    Montreal,
    Longueuil,
}

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
enum BritishColumbiaCity {
    #[default]
    Vancouver,
    Victoria,
}

#[derive(Clone, Debug, InfiniteSelect, PartialEq)]
enum UsaState {
    California(CaliforniaCity),
    Texas(TexasCity),
    NewYork(NewYorkCity),
}

impl Default for UsaState {
    fn default() -> Self {
        Self::California(CaliforniaCity::default())
    }
}

#[derive(Clone, Debug, InfiniteSelect, PartialEq)]
enum CanadaProvince {
    Ontario(OntarioCity),
    Quebec(QuebecCity),
    BritishColumbia(BritishColumbiaCity),
}

impl Default for CanadaProvince {
    fn default() -> Self {
        Self::Ontario(OntarioCity::default())
    }
}

#[derive(Clone, Debug, InfiniteSelect, PartialEq)]
enum Country {
    Usa(UsaState),
    Canada { province: CanadaProvince },
}

impl Default for Country {
    fn default() -> Self {
        Self::Usa(UsaState::default())
    }
}

#[derive(Clone, Debug, InfiniteSelect, PartialEq)]
enum PersistedSurface {
    #[tuple_enum(key = "web/app")]
    Web(PersistedRegion),
    #[tuple_enum(key = "docs/reference")]
    Docs,
}

impl Default for PersistedSurface {
    fn default() -> Self {
        Self::Web(PersistedRegion::default())
    }
}

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
enum PersistedRegion {
    #[default]
    #[tuple_enum(key = "us-east")]
    UsEast,
    #[tuple_enum(key = "eu\\west")]
    EuropeWest,
}

#[test]
fn depth_calculation_matches_nested_shape() {
    assert_eq!(CaliforniaCity::depth(), 1);
    assert_eq!(TexasCity::depth(), 1);
    assert_eq!(UsaState::depth(), 2);
    assert_eq!(CanadaProvince::depth(), 2);
    assert_eq!(Country::depth(), 3);
}

#[test]
fn variant_names_and_inner_flags_match_variants() {
    let usa = Country::Usa(UsaState::default());
    let california = UsaState::California(CaliforniaCity::default());

    assert_eq!(usa.variant_name(), "Usa");
    assert_eq!(california.variant_name(), "California");
    assert_eq!(CaliforniaCity::LosAngeles.variant_name(), "LosAngeles");
    assert!(usa.has_inner());
    assert!(!CaliforniaCity::LosAngeles.has_inner());
}

#[test]
fn child_variant_names_and_variants_are_reported() {
    let usa = Country::Usa(UsaState::default());
    let canada = Country::Canada {
        province: CanadaProvince::default(),
    };

    assert_eq!(
        usa.child_variant_names(),
        vec!["California", "Texas", "NewYork"]
    );
    assert_eq!(
        canada.child_variant_names(),
        vec!["Ontario", "Quebec", "BritishColumbia"]
    );

    let countries = Country::variants();
    assert_eq!(countries.len(), 2);
    assert_eq!(countries[0].variant_name(), "Usa");
    assert_eq!(countries[1].variant_name(), "Canada");
}

#[test]
fn set_child_by_index_updates_the_nested_value() {
    let updated = Country::Usa(UsaState::default())
        .set_child_by_index(1)
        .expect("Texas should be a valid child");

    match updated {
        Country::Usa(state) => assert_eq!(state.variant_name(), "Texas"),
        Country::Canada { .. } => panic!("expected Usa variant"),
    }
}

#[test]
fn build_from_index_path_recreates_nested_values() {
    let mut path = InfiniteSelectPath::new();
    path.set(0, 0);
    path.set(1, 1);
    path.set(2, 2);

    let country: Country = build_from_path(&path).expect("path should be valid");

    match country {
        Country::Usa(UsaState::Texas(city)) => assert_eq!(city.variant_name(), "Austin"),
        _ => panic!("expected Usa -> Texas -> Austin"),
    }
}

#[test]
fn selection_path_round_trips_the_current_value() {
    let value = Country::Canada {
        province: CanadaProvince::Quebec(QuebecCity::Longueuil),
    };

    let rebuilt: Country =
        build_from_path(&value.selection_path()).expect("selection path should rebuild");

    assert_eq!(rebuilt, value);
}

#[test]
fn build_from_key_path_recreates_nested_values() {
    let path = InfiniteSelectKeyPath::with_keys(vec![
        "Usa".to_string(),
        "Texas".to_string(),
        "Austin".to_string(),
    ]);

    let country: Country = build_from_key_path(&path).expect("key path should be valid");

    match country {
        Country::Usa(UsaState::Texas(city)) => assert_eq!(city.variant_key(), "Austin"),
        _ => panic!("expected Usa -> Texas -> Austin"),
    }
}

#[test]
fn selection_key_path_round_trips_the_current_value() {
    let value = Country::Canada {
        province: CanadaProvince::Quebec(QuebecCity::Longueuil),
    };

    let rebuilt: Country = build_from_key_path(&value.selection_key_path())
        .expect("selection key path should rebuild");

    assert_eq!(rebuilt, value);
}

#[test]
fn build_from_path_reports_typed_errors() {
    let error =
        build_from_path::<Country>(&InfiniteSelectPath::new()).expect_err("empty path should fail");
    assert_eq!(error.depth(), 0);
    assert_eq!(error.segment(), None);
    assert_eq!(error.reason(), &InfiniteSelectPathErrorReason::EmptyPath);

    let invalid_path = InfiniteSelectPath::with_indices(vec![0, 99]);
    let error = build_from_path::<Country>(&invalid_path).expect_err("invalid child should fail");
    assert_eq!(error.depth(), 1);
    assert_eq!(error.segment(), Some(&InfiniteSelectPathSegment::Index(99)));
    assert_eq!(
        error.reason(),
        &InfiniteSelectPathErrorReason::InvalidIndex { option_count: 3 }
    );
}

#[test]
fn build_from_key_path_reports_typed_errors() {
    let invalid_path = InfiniteSelectKeyPath::with_keys(vec![
        "Usa".to_string(),
        "Texas".to_string(),
        "MoonBase".to_string(),
    ]);
    let error = build_from_key_path::<Country>(&invalid_path).expect_err("unknown key should fail");

    assert_eq!(error.depth(), 2);
    assert_eq!(
        error.segment(),
        Some(&InfiniteSelectPathSegment::Key("MoonBase".to_string()))
    );
    let InfiniteSelectPathErrorReason::UnknownKey { available_keys } = error.reason() else {
        panic!("expected unknown-key error");
    };
    assert!(available_keys.iter().any(|key| key == "Austin"));
    assert!(available_keys.iter().any(|key| key == "Houston"));
}

#[test]
fn key_path_display_parse_and_serde_round_trip() {
    let value = PersistedSurface::Web(PersistedRegion::EuropeWest);
    let path = value.selection_key_path();

    assert_eq!(
        path.keys(),
        &["web/app".to_string(), "eu\\west".to_string()]
    );

    let encoded = path.to_string();
    assert_eq!(encoded, "web\\/app/eu\\\\west");

    let parsed: InfiniteSelectKeyPath = encoded.parse().expect("display output should parse");
    assert_eq!(parsed, path);

    let serialized = serde_json::to_string(&path).expect("key path should serialize");
    assert_eq!(serialized, "\"web\\\\/app/eu\\\\\\\\west\"");

    let deserialized: InfiniteSelectKeyPath =
        serde_json::from_str(&serialized).expect("key path should deserialize");
    assert_eq!(deserialized, path);
}

#[test]
fn custom_variant_keys_round_trip() {
    let value = PersistedSurface::Web(PersistedRegion::EuropeWest);

    assert_eq!(value.variant_key(), "web/app");
    assert_eq!(PersistedSurface::Docs.variant_key(), "docs/reference");

    let rebuilt = build_from_key_path::<PersistedSurface>(&value.selection_key_path())
        .expect("custom variant keys should rebuild");
    assert_eq!(rebuilt, value);
}

#[test]
fn child_depth_matches_next_nested_level() {
    assert_eq!(Country::Usa(UsaState::default()).child_depth(), 2);
    assert_eq!(
        UsaState::California(CaliforniaCity::default()).child_depth(),
        1
    );
    assert_eq!(CaliforniaCity::LosAngeles.child_depth(), 0);
}

#[test]
fn index_and_key_paths_support_mutation_and_empty_round_trips() {
    let mut path = InfiniteSelectPath::with_indices(vec![0, 1, 2]);
    assert_eq!(path.get(1), Some(1));
    assert_eq!(path.len(), 3);
    assert!(!path.is_empty());
    path.set(1, 9);
    assert_eq!(path.indices(), &[0, 9]);
    path.clear_from(1);
    assert_eq!(path.indices(), &[0]);
    path.truncate(0);
    assert!(path.is_empty());

    let mut keys = InfiniteSelectKeyPath::new();
    assert!(keys.is_empty());
    keys.set(0, "root");
    keys.set(1, "child");
    assert_eq!(keys.get(1), Some("child"));
    assert_eq!(keys.len(), 2);
    keys.clear_from(1);
    assert_eq!(keys.keys(), &["root".to_string()]);
    keys.set(1, "child");
    keys.truncate(1);
    assert_eq!(keys.to_string(), "root");
    assert_eq!(
        "".parse::<InfiniteSelectKeyPath>().unwrap(),
        InfiniteSelectKeyPath::new()
    );

    let error = "root\\".parse::<InfiniteSelectKeyPath>().unwrap_err();
    assert_eq!(error.input(), "root\\");
    assert!(error.to_string().contains("incomplete escape sequence"));
}

#[cfg(feature = "component-shape")]
#[gpui_kit::test]
fn infinite_select_items_expose_labels_values_and_child_selection(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.update(|cx| {
        let item = InfiniteSelectItem::from_variant(Country::default(), cx);
        assert_eq!(item.title().as_ref(), "Usa");
        assert_eq!(item.get_value().variant_name(), "Usa");
        assert!(item.has_inner());
        assert_eq!(
            item.child_variant_names(),
            vec!["California", "Texas", "NewYork"]
        );
        assert_eq!(
            item.child_variant_keys(),
            vec!["California", "Texas", "NewYork"]
        );
        assert_eq!(item.child_variant_labels(cx).len(), 3);

        let texas = item.with_child_at(1, cx).expect("Texas child should exist");
        assert_eq!(texas.title().as_ref(), "Texas");
        assert_eq!(
            texas.value().inner_child_variant_names(),
            vec!["Houston", "Dallas", "Austin"]
        );
        assert!(item.with_child_at(99, cx).is_none());
        assert_eq!(
            InfiniteSelectItem::new(Country::default(), "Custom")
                .title()
                .as_ref(),
            "Custom"
        );
        assert_eq!(item.into_value(), Country::default());

        let root_items = gpui_form_component::infinite_select::to_select_items::<Country>(cx);
        assert_eq!(root_items.len(), 2);
        assert_eq!(root_items[1].title().as_ref(), "Canada");
    });
}

#[cfg(feature = "component-shape")]
#[test]
fn infinite_select_options_default_to_non_searchable() {
    assert_eq!(
        InfiniteSelectOptions::builder().build(),
        InfiniteSelectOptions::default()
    );
}

#[cfg(feature = "component-shape")]
#[test]
fn infinite_select_options_record_searchable_configuration() {
    assert_eq!(
        InfiniteSelectOptions::builder().searchable(true).build(),
        InfiniteSelectOptions::new(true, None)
    );
    assert_eq!(
        InfiniteSelectComponent::<Country>::searchable(true),
        InfiniteSelectOptions::new(true, None)
    );
    assert_ne!(
        InfiniteSelectOptions::new(true, Some(3)),
        InfiniteSelectOptions::new(true, Some(2))
    );
}

#[cfg(feature = "component-shape")]
#[allow(dead_code)]
fn assert_infinite_select_options_build_shape<D>()
where
    D: SelectDelegate<Item = InfiniteSelectItem<Country>>
        + From<Vec<InfiniteSelectItem<Country>>>
        + 'static,
    InfiniteSelectOptions:
        component_shape_gpui::GpuiComponentShapeBuilder<InfiniteSelectComponent<Country, D>>,
{
    let _ = InfiniteSelectComponent::<Country, D>::from(
        InfiniteSelectOptions::builder().searchable(true).build(),
    );
    let _ = InfiniteSelectComponent::<Country, D>::from(InfiniteSelectOptions::new(true, Some(3)));
    let _ = InfiniteSelectComponent::<Country, D>::searchable(true);
}
