// SPDX-License-Identifier: Apache-2.0
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Display {
    #[default]
    Flex,
    Grid,
    Block,
    None,
}