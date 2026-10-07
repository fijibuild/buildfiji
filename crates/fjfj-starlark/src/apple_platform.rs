//! `apple_platform`, the type of `ctx.fragments.apple.single_arch_platform`
//! (buildfiji-d7l2): it prints as its name, is equal only to the platform of
//! that name, and has four members.
//!
//! `apple_common.platform.macos` and the rest are plain structs in Bazel 9.2.0
//! and stay so in `builtins.bzl`.

use allocative::Allocative;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::{NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::fmt;

/// `(name, name_in_plist, platform_type, is_device)` of each platform.
const PLATFORMS: [(&str, &str, &str, bool); 10] = [
    ("catalyst", "MacOSX", "catalyst", true),
    ("ios_device", "iPhoneOS", "ios", true),
    ("ios_simulator", "iPhoneSimulator", "ios", false),
    ("macos", "MacOSX", "macos", true),
    ("tvos_device", "AppleTVOS", "tvos", true),
    ("tvos_simulator", "AppleTVSimulator", "tvos", false),
    ("visionos_device", "XROS", "visionos", true),
    ("visionos_simulator", "XRSimulator", "visionos", false),
    ("watchos_device", "WatchOS", "watchos", true),
    ("watchos_simulator", "WatchSimulator", "watchos", false),
];

#[derive(Debug, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct ApplePlatform {
    index: usize,
}

starlark_simple_value!(ApplePlatform);

impl ApplePlatform {
    /// The platform called `name`, if there is one.
    pub(crate) fn named(name: &str) -> Option<ApplePlatform> {
        PLATFORMS
            .iter()
            .position(|p| p.0 == name)
            .map(|index| ApplePlatform { index })
    }

    fn row(&self) -> (&'static str, &'static str, &'static str, bool) {
        PLATFORMS[self.index]
    }
}

impl fmt::Display for ApplePlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.row().0)
    }
}

#[starlark_value(type = "apple_platform")]
impl<'v> StarlarkValue<'v> for ApplePlatform {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("apple_platform", apple_platform_members);
        Some(RES.methods())
    }

    fn write_hash(
        &self,
        hasher: &mut starlark::collections::StarlarkHasher,
    ) -> starlark::Result<()> {
        use std::hash::Hash;
        self.index.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other
            .downcast_ref::<ApplePlatform>()
            .is_some_and(|o| o.index == self.index))
    }
}

#[starlark_module]
fn apple_platform_members(builder: &mut MethodsBuilder) {
    #[starlark(attribute)]
    fn name(this: &ApplePlatform) -> starlark::Result<String> {
        Ok(this.row().0.to_owned())
    }

    #[starlark(attribute)]
    fn name_in_plist(this: &ApplePlatform) -> starlark::Result<String> {
        Ok(this.row().1.to_owned())
    }

    #[starlark(attribute)]
    fn platform_type(this: &ApplePlatform) -> starlark::Result<String> {
        Ok(this.row().2.to_owned())
    }

    #[starlark(attribute)]
    fn is_device(this: &ApplePlatform) -> starlark::Result<bool> {
        Ok(this.row().3)
    }
}
