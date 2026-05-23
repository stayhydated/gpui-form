use strum::{Display, EnumString, IntoStaticStr};

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentKind {
    Input,
    NumberInput,
    Checkbox,
    Switch,
    Select,
    InfiniteSelect,
    Custom,
    DatePicker,
    FilePicker,
}

impl ComponentKind {
    pub fn component_name(self) -> &'static str {
        self.into()
    }

    pub const fn is_value_only_field(self) -> bool {
        matches!(self, Self::Checkbox | Self::Switch)
    }

    pub const fn needs_value_field(self) -> bool {
        matches!(self, Self::NumberInput)
    }

    pub const fn subscribable(self) -> bool {
        matches!(
            self,
            Self::Input
                | Self::NumberInput
                | Self::Select
                | Self::InfiniteSelect
                | Self::DatePicker
                | Self::FilePicker
        )
    }

    pub const fn focusable(self) -> bool {
        matches!(
            self,
            Self::Input
                | Self::NumberInput
                | Self::Select
                | Self::InfiniteSelect
                | Self::FilePicker
        )
    }

    pub const fn default_wraps_in_option(self) -> bool {
        matches!(self, Self::Input | Self::NumberInput | Self::FilePicker)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectBehaviour {
    pub partial: bool,
    pub searchable: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectBehaviour {
    pub searchable: bool,
    pub max_depth: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NumberInputKind {
    #[default]
    Float,
    SignedInteger,
    UnsignedInteger,
    Custom,
}

impl NumberInputKind {
    pub const fn is_unsigned(self) -> bool {
        matches!(self, Self::UnsignedInteger)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NumberInputBehaviour {
    /// Optional `as = ...` override used for numeric validation semantics.
    pub validation_type: Option<&'static str>,
    pub kind: NumberInputKind,
}

#[derive(Clone, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentsBehaviour {
    Input,
    NumberInput(NumberInputBehaviour),
    Checkbox,
    Switch,
    Select(SelectBehaviour),
    InfiniteSelect(InfiniteSelectBehaviour),
    Custom,
    DatePicker,
    FilePicker,
}

impl ComponentsBehaviour {
    pub const fn kind(&self) -> ComponentKind {
        match self {
            Self::Input => ComponentKind::Input,
            Self::NumberInput(_) => ComponentKind::NumberInput,
            Self::Checkbox => ComponentKind::Checkbox,
            Self::Switch => ComponentKind::Switch,
            Self::Select(_) => ComponentKind::Select,
            Self::InfiniteSelect(_) => ComponentKind::InfiniteSelect,
            Self::Custom => ComponentKind::Custom,
            Self::DatePicker => ComponentKind::DatePicker,
            Self::FilePicker => ComponentKind::FilePicker,
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

    pub fn partial(&self) -> bool {
        match self {
            Self::Select(options) => options.partial,
            _ => false,
        }
    }

    pub const fn subscribable(&self) -> bool {
        self.kind().subscribable()
    }

    pub const fn focusable(&self) -> bool {
        self.kind().focusable()
    }
}
