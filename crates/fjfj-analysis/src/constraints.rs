//! A platform's constraints with the defaults it does not name (buildfiji-491h).
//!
//! A platform lists a value for some `constraint_setting`s. For any other, its
//! value is the setting's `default_constraint_value`, when it has one, so a
//! `target_compatible_with` of that value holds on it.

use crate::target::{ConfiguredTarget, ConfiguredTargetKey};
use fjfj_engine::{Ctx, Error};
use fjfj_graph::rule::AttrValue;
use fjfj_graph::{Configuration, Label};
use std::collections::BTreeSet;

/// `have` and, for each of `needed` that it lacks, that value when it is the
/// default of its setting and `have` has no value of that setting.
pub(crate) async fn with_defaults_for(
    ctx: &Ctx,
    have: &BTreeSet<Label>,
    needed: &[Label],
) -> Result<BTreeSet<Label>, Error> {
    let mut out = have.clone();
    for value in needed.iter().filter(|v| !have.contains(*v)) {
        if is_unnamed_default(ctx, have, value).await? {
            out.insert(value.clone());
        }
    }
    Ok(out)
}

async fn target_of(ctx: &Ctx, label: &Label) -> Result<std::sync::Arc<ConfiguredTarget>, Error> {
    ctx.get(ConfiguredTargetKey {
        label: label.clone(),
        configuration: Configuration::default(),
    })
    .await
}

async fn is_unnamed_default(
    ctx: &Ctx,
    have: &BTreeSet<Label>,
    value: &Label,
) -> Result<bool, Error> {
    // A label that is not a constraint_value is not the default of anything;
    // the platform lacks it.
    let Ok(target) = target_of(ctx, value).await else {
        return Ok(false);
    };
    let Some(setting) = &target.constraint_setting else {
        return Ok(false);
    };
    let setting_target = target_of(ctx, setting).await?;
    if default_of(&setting_target).as_ref() != Some(value) {
        return Ok(false);
    }
    for member in have {
        if let Ok(m) = target_of(ctx, member).await
            && m.constraint_setting.as_ref() == Some(setting)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The `default_constraint_value` of a `constraint_setting`, a name in its
/// package.
fn default_of(setting: &ConfiguredTarget) -> Option<Label> {
    let name = setting.attrs.iter().find_map(|(n, v)| match v {
        AttrValue::String(s) if n == "default_constraint_value" && !s.is_empty() => Some(s),
        _ => None,
    })?;
    Some(Label {
        repo: setting.label.repo.clone(),
        package: setting.label.package.clone(),
        name: name.trim_start_matches(':').to_owned(),
    })
}
