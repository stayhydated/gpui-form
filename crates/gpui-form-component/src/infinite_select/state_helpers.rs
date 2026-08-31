use super::*;

pub(super) fn build_child_selects<T, D>(
    parent: &T,
    path: &InfiniteSelectPath,
    max_depth: usize,
    searchable: bool,
    window: &mut Window,
    cx: &mut Context<InfiniteSelectState<T, D>>,
) -> Vec<Entity<SelectState<D>>>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    let mut current_value = parent.clone();
    let mut selects = Vec::new();

    for level in 0..max_depth.saturating_sub(1) {
        let items = child_items_for_level(&current_value, level, cx);
        if items.is_empty() {
            break;
        }

        let selected_row = path.get(level + 1).unwrap_or(0).min(items.len() - 1);
        let child_select = cx.new(|cx| {
            build_select_state::<T, D>(items.clone(), Some(selected_row), searchable, window, cx)
        });
        current_value = items[selected_row].get_value().clone();
        selects.push(child_select);
    }

    selects
}

pub(super) fn build_levels<T, D>(
    value: &T,
    path: &InfiniteSelectPath,
    key_path: &InfiniteSelectKeyPath,
    master_select: &Entity<SelectState<D>>,
    child_selects: &[Entity<SelectState<D>>],
    cx: &impl std::borrow::Borrow<App>,
) -> Vec<InfiniteSelectLevel<D>>
where
    T: InfiniteSelectValue,
    D: SelectDelegate + 'static,
{
    let mut levels = Vec::with_capacity(child_selects.len() + 1);
    levels.push(InfiniteSelectLevel {
        depth: 0,
        label: value.type_label(cx),
        description: value.type_description(cx),
        select: master_select.clone(),
        selected_index: path.get(0),
        selected_key: key_path.get(0).map(str::to_string),
    });

    levels.extend(child_selects.iter().enumerate().map(|(index, select)| {
        let depth = index + 1;
        InfiniteSelectLevel {
            depth,
            label: value.child_label_at_depth(index, cx).unwrap_or_else(|| {
                panic!("missing infinite-select label metadata at depth {depth}")
            }),
            description: value
                .child_description_at_depth(index, cx)
                .unwrap_or_else(|| {
                    panic!("missing infinite-select description metadata at depth {depth}")
                }),
            select: select.clone(),
            selected_index: path.get(depth),
            selected_key: key_path.get(depth).map(str::to_string),
        }
    }));

    levels
}

fn child_items_for_level<T: InfiniteSelectValue>(
    current_value: &T,
    level: usize,
    cx: &impl std::borrow::Borrow<App>,
) -> Vec<InfiniteSelectItem<T>> {
    let (has_more, child_labels) = if level == 0 {
        (
            current_value.has_inner(),
            current_value.child_variant_labels(cx),
        )
    } else {
        (
            current_value.inner_has_inner(),
            current_value.inner_child_variant_labels(cx),
        )
    };

    if !has_more || child_labels.is_empty() {
        return Vec::new();
    }

    child_labels
        .into_iter()
        .enumerate()
        .filter_map(|(index, title)| {
            let value = if level == 0 {
                current_value.set_child_by_index(index)
            } else {
                current_value.inner_set_child_by_index(index)
            };
            value.map(|value| InfiniteSelectItem::new(value, title))
        })
        .collect()
}

pub(super) fn build_select_state<T, D>(
    items: Vec<InfiniteSelectItem<T>>,
    selected_row: Option<usize>,
    searchable: bool,
    window: &mut Window,
    cx: &mut Context<SelectState<D>>,
) -> SelectState<D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    let mut state = SelectState::new(
        items.into(),
        selected_row.and_then(selected_index),
        window,
        cx,
    );
    if searchable {
        state = state.searchable(true);
    }
    state
}

pub(super) fn selected_index(row: usize) -> Option<IndexPath> {
    Some(IndexPath {
        section: 0,
        row,
        column: 0,
    })
}

pub(super) fn first_changed_depth(
    previous: &InfiniteSelectPath,
    next: &InfiniteSelectPath,
) -> usize {
    let max_depth = previous.len().max(next.len());
    for depth in 0..max_depth {
        if previous.get(depth) != next.get(depth) {
            return depth;
        }
    }

    max_depth.saturating_sub(1)
}
