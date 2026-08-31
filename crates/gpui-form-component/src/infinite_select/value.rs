use super::*;

pub trait InfiniteSelectValue: Sized + Clone + Default + PartialEq + 'static {
    /// Returns all possible variants at this level with default inner values.
    fn variants() -> Vec<Self>;

    /// Returns the variant name/discriminant as a string.
    fn variant_name(&self) -> &'static str;

    /// Returns a stable key for this variant.
    ///
    /// The derived implementation currently mirrors `variant_name()`, which is
    /// stable across enum reordering and suitable for persisted paths.
    fn variant_key(&self) -> &'static str;

    /// Returns the localized label for this specific variant.
    fn variant_label(&self, cx: &impl std::borrow::Borrow<App>) -> SharedString;

    /// Returns true if this variant contains an inner value.
    fn has_inner(&self) -> bool;

    /// Returns the variant names of the children for this specific variant.
    /// Returns an empty vec for unit variants or variants without children.
    fn child_variant_names(&self) -> Vec<&'static str>;

    /// Returns the stable keys of the children for this specific variant.
    fn child_variant_keys(&self) -> Vec<&'static str>;

    /// Returns the localized labels of the children for this specific variant.
    fn child_variant_labels(&self, cx: &impl std::borrow::Borrow<App>) -> Vec<SharedString>;

    /// Creates a new instance with the child at the given index.
    /// Returns None if the variant doesn't have children or the index is out of bounds.
    fn set_child_by_index(&self, index: usize) -> Option<Self>;

    /// Creates a new instance with the child that matches the given key.
    fn set_child_by_key(&self, key: &str) -> Option<Self>;

    /// Sets the child at a given path depth recursively.
    /// `path[0]` is the child index at this level, `path[1]` the grandchild, and so on.
    fn set_child_by_path(&self, path: &[usize]) -> Option<Self>;

    /// Sets the child at a given key path recursively.
    fn set_child_by_key_path(&self, path: &[String]) -> Option<Self>;

    /// Returns the depth of nesting for this variant's children.
    /// Returns 0 for leaf variants.
    fn child_depth(&self) -> usize;

    /// Returns the maximum depth of nesting for this enum type.
    fn depth() -> usize;

    /// Returns the current selection path for this concrete value.
    ///
    /// The returned path includes the root variant index at depth 0.
    fn selection_path(&self) -> InfiniteSelectPath;

    /// Returns the current stable key path for this concrete value.
    fn selection_key_path(&self) -> InfiniteSelectKeyPath;

    /// Returns the variant names of the inner value's children.
    fn inner_child_variant_names(&self) -> Vec<&'static str>;

    /// Returns the stable keys of the inner value's children.
    fn inner_child_variant_keys(&self) -> Vec<&'static str>;

    /// Returns the localized labels of the inner value's children.
    fn inner_child_variant_labels(&self, cx: &impl std::borrow::Borrow<App>) -> Vec<SharedString>;

    /// Sets a child on the inner value and wraps it back.
    fn inner_set_child_by_index(&self, index: usize) -> Option<Self>;

    /// Sets a child on the inner value by stable key and wraps it back.
    fn inner_set_child_by_key(&self, key: &str) -> Option<Self>;

    /// Returns true if the inner value itself has children.
    fn inner_has_inner(&self) -> bool;

    /// Returns the localized label for this type (level).
    fn type_label(&self, cx: &impl std::borrow::Borrow<App>) -> SharedString;

    /// Returns the localized description for this type (level).
    fn type_description(&self, cx: &impl std::borrow::Borrow<App>) -> SharedString;

    /// Returns the localized label for the child at the given depth relative to this node.
    /// `depth = 0` is the immediate child.
    fn child_label_at_depth(
        &self,
        depth: usize,
        cx: &impl std::borrow::Borrow<App>,
    ) -> Option<SharedString>;

    /// Returns the localized description for the child at the given depth.
    fn child_description_at_depth(
        &self,
        depth: usize,
        cx: &impl std::borrow::Borrow<App>,
    ) -> Option<SharedString>;

    /// Internal method to delegate label lookup to the inner value.
    fn inner_child_label_at_depth(
        &self,
        depth: usize,
        cx: &impl std::borrow::Borrow<App>,
    ) -> Option<SharedString>;

    /// Internal method to delegate description lookup to the inner value.
    fn inner_child_description_at_depth(
        &self,
        depth: usize,
        cx: &impl std::borrow::Borrow<App>,
    ) -> Option<SharedString>;
}

/// A wrapper for infinite-select enum variants that implements `SelectItem`.
///
/// This allows infinite-select enum variants to be displayed in a select dropdown
/// while preserving access to the nested value.
#[derive(Clone)]
pub struct InfiniteSelectItem<T: InfiniteSelectValue> {
    value: T,
    title: SharedString,
}

impl<T: InfiniteSelectValue> InfiniteSelectItem<T> {
    /// Creates a new item with a custom title.
    pub fn new(value: T, title: impl Into<SharedString>) -> Self {
        Self {
            value,
            title: title.into(),
        }
    }

    /// Creates an item using `variant_label(cx)` as the title.
    pub fn from_variant(value: T, cx: &impl std::borrow::Borrow<App>) -> Self {
        let title = value.variant_label(cx);
        Self { value, title }
    }

    /// Returns a reference to the wrapped value.
    pub fn get_value(&self) -> &T {
        &self.value
    }

    /// Consumes the item and returns the wrapped value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns true if the wrapped value has nested inner values.
    pub fn has_inner(&self) -> bool {
        self.value.has_inner()
    }

    /// Returns the child variant names if the wrapped value has children.
    pub fn child_variant_names(&self) -> Vec<&'static str> {
        self.value.child_variant_names()
    }

    /// Returns the child variant keys if the wrapped value has children.
    pub fn child_variant_keys(&self) -> Vec<&'static str> {
        self.value.child_variant_keys()
    }

    /// Returns the child variant labels if the wrapped value has children.
    pub fn child_variant_labels(&self, cx: &impl std::borrow::Borrow<App>) -> Vec<SharedString> {
        self.value.child_variant_labels(cx)
    }

    /// Returns a new item with a child selected at the given index.
    pub fn with_child_at(&self, index: usize, cx: &impl std::borrow::Borrow<App>) -> Option<Self> {
        let title = self.value.child_variant_labels(cx).get(index).cloned()?;
        self.value
            .set_child_by_index(index)
            .map(|value| Self::new(value, title))
    }
}

impl<T: InfiniteSelectValue> SelectItem for InfiniteSelectItem<T> {
    type Value = T;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// Creates root select items from `T::variants()`.
pub fn to_select_items<T>(cx: &impl std::borrow::Borrow<App>) -> Vec<InfiniteSelectItem<T>>
where
    T: InfiniteSelectValue,
{
    T::variants()
        .into_iter()
        .map(|value| InfiniteSelectItem::from_variant(value, cx))
        .collect()
}
