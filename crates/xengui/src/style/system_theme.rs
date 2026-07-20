// SPDX-License-Identifier: Apache-2.0

/// OS/host light-dark appearance, independent of any windowing backend.
/// A platform layer (xenframe, a Bevy plugin, etc.) converts its own
/// theme type into this before calling into xengui's renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SystemTheme {
    Light,
    #[default]
    Dark,
}
