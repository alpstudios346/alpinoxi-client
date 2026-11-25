//! The page the launcher shows, as one string.
//!
//! Built from `ui/hudui.html` by `ui/build_ui.py`; the Rust side only serves
//! it, so a change to the interface never needs a change here.

pub const INDEX_HTML: &str = include_str!("../../../ui/dist/alpinoxi-ui.html");
