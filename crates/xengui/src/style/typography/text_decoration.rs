#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TextDecoration {
    underline: bool,
    strike: bool,
    overline: bool,
}