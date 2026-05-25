use strum::{Display, EnumString, IntoStaticStr};

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentsBehaviour {
    Shape,
}

impl ComponentsBehaviour {
    pub fn component_name(&self) -> &'static str {
        (*self).into()
    }
}
