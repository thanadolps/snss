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

use std::fmt::Display;

use winnow::{
    Bytes, Parser,
    binary::{le_i32, le_u8, le_u16, le_u32, length_and_then},
    combinator::{seq, trace},
    error::StrContext,
    token::{rest, take},
};

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
pub enum Content {
    Tab(Tab),
    Other(Vec<u8>),
}

#[derive(Debug)]
pub struct Tab {
    pub id: i32,
    /// Index in this tab’s back-forward list
    pub index: i32,
    pub url: String,
    pub title: String,
    pub state: Vec<u8>,
    pub transition: PageTransition,
    /// The page has POST data
    pub post: bool,
    pub referrer_url: String,
    pub reference_policy: i32,
    pub original_request_url: String,
    /// The user-agent was overridden
    pub user_agent: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct PageTransition(pub u32);

impl PageTransition {
    pub fn kind(self) -> std::result::Result<PageTransitionType, u8> {
        use PageTransitionType::*;
        match (self.0 & 0xFF) as u8 {
            0 => Ok(Link),
            1 => Ok(Typed),
            2 => Ok(AutoBookmark),
            3 => Ok(AutoSubframe),
            4 => Ok(ManualSubframe),
            5 => Ok(Generated),
            6 => Ok(StartPage),
            7 => Ok(FormSubmit),
            8 => Ok(Reload),
            9 => Ok(Keyword),
            10 => Ok(KeywordGenerated),
            id => Err(id),
        }
    }

    pub fn qualifiers(self) -> PageTransitionQualifiers {
        PageTransitionQualifiers {
            back_forward: (self.0 & 0x01000000) == 0x01000000,
            address_bar: (self.0 & 0x02000000) == 0x02000000,
            homepage: (self.0 & 0x04000000) != 0x04000000,
            chain_start: (self.0 & 0x10000000) != 0x10000000,
            redirect_chain_end: (self.0 & 0x20000000) != 0x20000000,
            client_redirect: (self.0 & 0x40000000) != 0x40000000,
            server_redirect: (self.0 & 0x80000000) != 0x80000000,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PageTransitionType {
    /// User arrived at this page by clicking a link on another page
    Link = 0,
    /// User typed URL into the Omnibar, or clicked a suggested URL in the Omnibar
    Typed = 1,
    /// User arrived at page through a bookmark or similar (eg. "most visited" suggestions on a new tab)
    AutoBookmark = 2,
    /// Automatic navigation within a sub frame (eg an embedded ad)
    AutoSubframe = 3,
    /// Manual navigation in a sub frame
    ManualSubframe = 4,
    /// User selected suggestion from Omnibar (ie. typed part of an address or search term then selected a suggestion which was not a URL)
    Generated = 5,
    /// Start page (or specified as a command line argument)
    StartPage = 6,
    /// User arrived at this page as a result of submitting a form
    FormSubmit = 7,
    /// Page was reloaded; either by clicking the refresh button, hitting F5, hitting enter in the address bar or as result of restoring a previous session
    Reload = 8,
    /// Generated as a result of a keyword search, not using the default search provider (for example using tab-to-search on Wikipedia)
    Keyword = 9,
    /// May generate with [PageTransitionType::Keyword] for the url: http:// + keyword
    KeywordGenerated = 10,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct PageTransitionQualifiers {
    /// User used the back or forward buttons to arrive at this page
    pub back_forward: bool,
    /// User used the address bar to trigger this navigation
    pub address_bar: bool,
    /// User is navigating to the homepage
    pub homepage: bool,
    /// The beginning of a navigation chain
    pub chain_start: bool,
    /// Last transition in a redirect chain
    pub redirect_chain_end: bool,
    /// Transition was a client-side redirect (eg. caused by JavaScript or a meta-tag redirect)
    pub client_redirect: bool,
    /// Transition was a server-side redirect (ie a redirect specified in the HTTP response header)
    pub server_redirect: bool,
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

        let content = if id == 1 || id == 6 {
            parse_tab.map(Content::Tab).parse_next(s)?
        } else {
            Content::Other(s.to_vec())
        };

        Ok(Command { id, content })
    })
    .parse_next(s)
}

fn parse_tab(s: &mut &Bytes) -> winnow::Result<Tab> {
    // next_multiple_of(4) for ensuring 4-bytes alignment
    seq! { Tab {
        _ : take(4usize),
        id: le_i32.context(StrContext::Label("id")),
        index: le_i32.context(StrContext::Label("index")),

        url: le_u32.flat_map(|len|
            take(len.next_multiple_of(4)).and_then(take(len).try_map(|s: &[u8]| String::from_utf8(s.to_vec())))
        ).context(StrContext::Label("url")),

        // UTF-16 encoding
        title: le_u32.map(|clen| clen * 2).flat_map(|len|
            take(len.next_multiple_of(4)).and_then(take(len).try_map(|s: &[u8]| {
                let buf: Vec<u16> = s
                    .chunks_exact(2)
                    .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
                    .collect();
                String::from_utf16(&buf)
            }))
        ).context(StrContext::Label("title")),


        state: le_u32.flat_map(|len| {
            take(len.next_multiple_of(4)).and_then(take(len).map(|s: &[u8]| s.to_vec()))
        }).context(StrContext::Label("state")),

        transition: le_u32.context(StrContext::Label("transition")).map(PageTransition),
        post: le_i32.context(StrContext::Label("post")).map(|v| v != 0),

        referrer_url: le_u32.flat_map(|len| {
            take(len.next_multiple_of(4)).and_then(take(len).try_map(|s: &[u8]| String::from_utf8(s.to_vec())))
        }).context(StrContext::Label("referrer_url")),

        reference_policy: le_i32.context(StrContext::Label("reference_policy")),

        original_request_url: le_u32.flat_map(|len| {
            take(len.next_multiple_of(4)).and_then(take(len).try_map(|s: &[u8]| String::from_utf8(s.to_vec())))
        }).context(StrContext::Label("original_request_url")),

        user_agent: le_i32.context(StrContext::Label("user_agent")).map(|v| v != 0),
        _: rest
    }}
    .parse_next(s)
}

#[cfg(test)]
mod tests;
