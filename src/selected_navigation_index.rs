use winnow::{Bytes, Parser, binary::le_i32, combinator::seq, error::StrContext, token::rest};

/// Which entry of a tab's back-forward list is currently shown
#[derive(Debug)]
pub struct SelectedNavigationIndex {
    pub tab_id: i32,
    pub index: i32,
}

pub(crate) fn parse_selected_navigation_index(
    s: &mut &Bytes,
) -> winnow::Result<SelectedNavigationIndex> {
    seq! { SelectedNavigationIndex {
        tab_id: le_i32.context(StrContext::Label("tab_id")),
        index: le_i32.context(StrContext::Label("index")),
        _: rest
    }}
    .parse_next(s)
}
