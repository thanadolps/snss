use winnow::{
    Bytes, Parser,
    binary::{le_i32, le_i64},
    combinator::seq,
    error::StrContext,
    token::{rest, take},
};

/// A tab was closed
#[derive(Debug)]
pub struct TabClosed {
    pub tab_id: i32,
    /// Microseconds since 1601-01-01 UTC
    pub close_time: i64,
}

pub(crate) fn parse_tab_closed(s: &mut &Bytes) -> winnow::Result<TabClosed> {
    seq! { TabClosed {
        tab_id: le_i32.context(StrContext::Label("tab_id")),
        // alignment padding before the 8-byte close_time
        _: take(4usize),
        close_time: le_i64.context(StrContext::Label("close_time")),
        _: rest
    }}
    .parse_next(s)
}
