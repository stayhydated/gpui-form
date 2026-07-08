//! Example of nested infinite select enums for location selection.
//!
//! This demonstrates a 3-level hierarchy: Country -> State/Province -> City
//! using the InfiniteSelect derive macro.

use es_fluent::{EsFluent, EsFluentLabel, EsFluentVariants};
use gpui_form_component::InfiniteSelect;
use strum::EnumIter;

// ============================================================================
// Level 3: Cities (leaf nodes - no inner values)
// ============================================================================

#[derive(
    Clone, Debug, Default, EnumIter, EsFluent, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
pub enum CaliforniaCity {
    #[default]
    LosAngeles,
    SanFrancisco,
    SanDiego,
    SanJose,
    Sacramento,
}

#[derive(
    Clone, Debug, Default, EnumIter, EsFluent, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
pub enum TexasCity {
    #[default]
    Houston,
    Dallas,
    Austin,
    SanAntonio,
    FortWorth,
}

#[derive(
    Clone, Debug, Default, EnumIter, EsFluent, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
pub enum NewYorkCity {
    #[default]
    NewYorkCity,
    Buffalo,
    Rochester,
    Albany,
    Syracuse,
}

#[derive(
    Clone, Debug, Default, EnumIter, EsFluent, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
pub enum OntarioCity {
    #[default]
    Toronto,
    Ottawa,
    Mississauga,
    Hamilton,
    London,
}

#[derive(
    Clone, Debug, Default, EnumIter, EsFluent, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
pub enum QuebecCity {
    #[default]
    Montreal,
    QuebecCity,
    Laval,
    Gatineau,
    Longueuil,
}

#[derive(
    Clone, Debug, Default, EnumIter, EsFluent, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
pub enum BritishColumbiaCity {
    #[default]
    Vancouver,
    Victoria,
    Surrey,
    Burnaby,
    Richmond,
}

// ============================================================================
// Level 2: States/Provinces (contain cities)
// ============================================================================

#[derive(
    Clone, Debug, EnumIter, EsFluent, EsFluentVariants, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
#[fluent_variants(keys = ["description", "label"])]
pub enum USAState {
    California(CaliforniaCity),
    Texas(TexasCity),
    NewYork(NewYorkCity),
}

impl Default for USAState {
    fn default() -> Self {
        Self::California(CaliforniaCity::default())
    }
}

#[derive(
    Clone, Debug, EnumIter, EsFluent, EsFluentVariants, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
#[fluent_variants(keys = ["description", "label"])]
pub enum CanadaProvince {
    Ontario(OntarioCity),
    Quebec(QuebecCity),
    BritishColumbia(BritishColumbiaCity),
}

impl Default for CanadaProvince {
    fn default() -> Self {
        Self::Ontario(OntarioCity::default())
    }
}

// ============================================================================
// Level 1: Countries (contain states/provinces)
// ============================================================================

#[derive(
    Clone, Debug, EnumIter, EsFluent, EsFluentVariants, EsFluentLabel, InfiniteSelect, Eq, PartialEq,
)]
#[fluent_variants(keys = ["description", "label"])]
pub enum Country {
    USA(USAState),
    Canada { province: CanadaProvince },
}

impl Default for Country {
    fn default() -> Self {
        Self::USA(USAState::default())
    }
}

// ============================================================================
// Form struct using the nested infinite select enum
// ============================================================================

use gpui_form::GpuiForm;

/// A form that demonstrates tuple select with nested enums.
#[derive(Clone, Debug, Default, EsFluentLabel, EsFluentVariants, GpuiForm)]
#[fluent_variants(keys = ["description", "label"])]
pub struct LocationForm {
    /// User's name
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub name: String,

    /// Location selection using cascading selects
    #[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>.searchable(true)))]
    pub location: Country,
}
