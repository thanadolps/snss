//! Basic SNSS file parsing (eg. Chrome Session and Tabs Files)
//!
//! # Examples
//! ```no_run
//! let data = std::fs::read("Session")?;
//! let snss = snss::parse(&data)?;
//! for command in snss.commands {
//!     if let snss::Content::Tab(tab) = command.content {
//!         println!("Tab #{}: [{}]({})", tab.id, tab.title, tab.url);
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod selected_navigation_index;
mod tab;

use std::fmt::Display;

use winnow::{
    Bytes, Parser,
    binary::{le_i32, le_u8, le_u16, length_and_then},
    combinator::{seq, trace},
};

pub use selected_navigation_index::SelectedNavigationIndex;
use selected_navigation_index::parse_selected_navigation_index;
use tab::parse_tab;
pub use tab::{PageTransition, PageTransitionQualifiers, PageTransitionType, Tab};

// Thanks for the following sources:
// - https://digitalinvestigation.wordpress.com/tag/snss
// - https://github.com/phacoxcll/SNSS_Reader
// - https://github.com/chromium/chromium/blob/main/ui/base/page_transition_types.h

#[derive(Debug)]
pub struct Error {
    message: String,
    offset: usize,
}
impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "error at offset {}: {}", self.offset, self.message)
    }
}

pub fn parse(data: &[u8]) -> Result<SNSS, Error> {
    parse_snss.parse(Bytes::new(data)).map_err(|err| Error {
        offset: err.offset(),
        message: err.into_inner().to_string(),
    })
}

#[derive(Debug)]
pub struct SNSS {
    pub version: i32,
    pub commands: Vec<Command>,
}

#[derive(Debug)]
pub struct Command {
    pub id: u8,
    pub content: Content,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Content {
    Tab(Tab),
    SelectedNavigationIndex(SelectedNavigationIndex),
    Other(Vec<u8>),
}

fn parse_snss(s: &mut &Bytes) -> winnow::Result<SNSS> {
    seq! { SNSS {
        _: b"SNSS",
        version: le_i32,
        commands: winnow::combinator::repeat(0.., length_and_then(le_u16, parse_command)),

    }}
    .parse_next(s)
}

fn parse_command<'s>(s: &mut &'s Bytes) -> winnow::Result<Command> {
    trace("Command", |s: &mut &'s Bytes| {
        let id = le_u8.parse_next(s)?;

        let content = match id {
            1 | 6 => parse_tab.map(Content::Tab).parse_next(s)?,
            7 => parse_selected_navigation_index
                .map(Content::SelectedNavigationIndex)
                .parse_next(s)?,
            _ => Content::Other(s.to_vec()),
        };

        Ok(Command { id, content })
    })
    .parse_next(s)
}

#[cfg(test)]
mod tests;
