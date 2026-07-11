const YEARS_PER_PAGE: usize = 20;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct CalendarNavigation {
    current_year: i32,
    current_month: u8,
    year_pages: Vec<Vec<i32>>,
    year_page: usize,
}

impl CalendarNavigation {
    pub(super) fn new(current_year: i32, current_month: u8, year_range: (i32, i32)) -> Self {
        debug_assert!((1..=12).contains(&current_month));
        let mut navigation = Self {
            current_year,
            current_month,
            year_pages: Vec::new(),
            year_page: 0,
        };
        navigation.set_year_range(year_range);
        navigation
    }

    pub(super) fn current_year(&self) -> i32 {
        self.current_year
    }

    pub(super) fn current_month(&self) -> u8 {
        self.current_month
    }

    pub(super) fn set_position(&mut self, year: i32, month: u8) {
        debug_assert!((1..=12).contains(&month));
        self.current_year = year;
        self.current_month = month;
        self.select_page_for_current_year();
    }

    pub(super) fn set_year(&mut self, year: i32) {
        self.current_year = year;
        self.select_page_for_current_year();
    }

    pub(super) fn set_month(&mut self, month: u8) {
        debug_assert!((1..=12).contains(&month));
        self.current_month = month;
    }

    pub(super) fn set_year_range(&mut self, range: (i32, i32)) {
        self.year_pages = (range.0..range.1)
            .collect::<Vec<_>>()
            .chunks(YEARS_PER_PAGE)
            .map(<[i32]>::to_vec)
            .collect();
        self.select_page_for_current_year();
    }

    fn select_page_for_current_year(&mut self) {
        self.year_page = self
            .year_pages
            .iter()
            .position(|years| years.contains(&self.current_year))
            .unwrap_or(0);
    }

    pub(super) fn offset_year_month(&self, offset_month: usize) -> (i32, u32) {
        let month_index =
            i64::from(self.current_month - 1) + i64::try_from(offset_month).unwrap_or(i64::MAX);
        let year_offset = month_index.div_euclid(12);
        let year = i64::from(self.current_year).saturating_add(year_offset);
        let year = i32::try_from(year).unwrap_or(i32::MAX);
        let month = month_index.rem_euclid(12) as u32 + 1;
        (year, month)
    }

    pub(super) fn has_previous_year_page(&self) -> bool {
        self.year_page > 0
    }

    pub(super) fn has_next_year_page(&self) -> bool {
        self.year_page + 1 < self.year_pages.len()
    }

    pub(super) fn previous_year_page(&mut self) -> bool {
        if !self.has_previous_year_page() {
            return false;
        }
        self.year_page -= 1;
        true
    }

    pub(super) fn next_year_page(&mut self) -> bool {
        if !self.has_next_year_page() {
            return false;
        }
        self.year_page += 1;
        true
    }

    pub(super) fn previous_month(&mut self) {
        if self.current_month == 1 {
            self.current_month = 12;
            self.current_year = self.current_year.saturating_sub(1);
            self.select_page_for_current_year();
        } else {
            self.current_month -= 1;
        }
    }

    pub(super) fn next_month(&mut self) {
        if self.current_month == 12 {
            self.current_month = 1;
            self.current_year = self.current_year.saturating_add(1);
            self.select_page_for_current_year();
        } else {
            self.current_month += 1;
        }
    }

    pub(super) fn current_page_years(&self) -> &[i32] {
        self.year_pages
            .get(self.year_page)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::CalendarNavigation;

    #[test]
    fn month_navigation_rolls_years_and_supports_offsets() {
        let mut navigation = CalendarNavigation::new(2025, 1, (2000, 2040));
        navigation.previous_month();
        assert_eq!(
            (navigation.current_year(), navigation.current_month()),
            (2024, 12)
        );
        navigation.previous_month();
        assert_eq!(
            (navigation.current_year(), navigation.current_month()),
            (2024, 11)
        );

        navigation.next_month();
        assert_eq!(
            (navigation.current_year(), navigation.current_month()),
            (2024, 12)
        );
        navigation.next_month();
        assert_eq!(
            (navigation.current_year(), navigation.current_month()),
            (2025, 1)
        );
        assert_eq!(navigation.offset_year_month(0), (2025, 1));
        assert_eq!(navigation.offset_year_month(11), (2025, 12));
        assert_eq!(navigation.offset_year_month(12), (2026, 1));
        assert_eq!(navigation.offset_year_month(25), (2027, 2));
    }

    #[test]
    fn year_pages_track_the_selected_year_and_stop_at_boundaries() {
        let mut navigation = CalendarNavigation::new(2025, 6, (2000, 2050));
        assert_eq!(
            navigation.current_page_years(),
            &(2020..2040).collect::<Vec<_>>()
        );
        assert!(navigation.has_previous_year_page());
        assert!(navigation.has_next_year_page());

        assert!(navigation.previous_year_page());
        assert_eq!(
            navigation.current_page_years(),
            &(2000..2020).collect::<Vec<_>>()
        );
        assert!(!navigation.previous_year_page());

        assert!(navigation.next_year_page());
        assert!(navigation.next_year_page());
        assert_eq!(
            navigation.current_page_years(),
            &(2040..2050).collect::<Vec<_>>()
        );
        assert!(!navigation.next_year_page());

        navigation.set_year(2002);
        assert_eq!(
            navigation.current_page_years(),
            &(2000..2020).collect::<Vec<_>>()
        );
    }

    #[test]
    fn position_and_range_changes_reselect_the_matching_page() {
        let mut navigation = CalendarNavigation::new(2025, 6, (2000, 2060));
        navigation.set_position(2055, 11);
        assert_eq!(navigation.current_year(), 2055);
        assert_eq!(navigation.current_month(), 11);
        assert_eq!(
            navigation.current_page_years(),
            &(2040..2060).collect::<Vec<_>>()
        );

        navigation.set_month(3);
        assert_eq!(navigation.current_month(), 3);
        navigation.set_year_range((2050, 2056));
        assert_eq!(
            navigation.current_page_years(),
            &(2050..2056).collect::<Vec<_>>()
        );

        navigation.set_year_range((0, 0));
        assert!(navigation.current_page_years().is_empty());
        assert!(!navigation.has_previous_year_page());
        assert!(!navigation.has_next_year_page());
    }
}
