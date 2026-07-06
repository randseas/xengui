// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TextDecoration {
    underline: bool,
    strike: bool,
    overline: bool,
}