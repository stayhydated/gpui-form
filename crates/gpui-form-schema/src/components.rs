use strum::{Display, EnumString, IntoStaticStr};

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentKind {
    Custom,
}

impl ComponentKind {
    pub fn component_name(self) -> &'static str {
        self.into()
    }

    pub const fn is_value_only_field(self) -> bool {
        false
    }

    pub const fn needs_value_field(self) -> bool {
        false
    }

    pub const fn subscribable(self) -> bool {
        false
    }

    pub const fn focusable(self) -> bool {
        false
    }

    pub const fn default_wraps_in_option(self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentsBehaviour {
    Custom,
}

impl ComponentsBehaviour {
    pub const fn kind(&self) -> ComponentKind {
        match self {
            Self::Custom => ComponentKind::Custom,
        }
    }

    pub fn component_name(&self) -> &'static str {
        self.kind().component_name()
    }

    pub const fn is_value_only_field(&self) -> bool {
        self.kind().is_value_only_field()
    }

    pub const fn needs_value_field(&self) -> bool {
        self.kind().needs_value_field()
    }

    pub const fn partial(&self) -> bool {
        false
    }

    pub const fn subscribable(&self) -> bool {
        self.kind().subscribable()
    }

    pub const fn focusable(&self) -> bool {
        self.kind().focusable()
    }
}
