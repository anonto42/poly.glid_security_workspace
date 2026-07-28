#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResizeAxis {
    Sidebar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LoadState {
    Loading,
    Empty,
    Ready,
    Error(String),
}
