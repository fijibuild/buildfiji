# The C++ feature engine (buildfiji-136.16.2): what Bazel's CcToolchainFeatures
# does in Java. It is given the features and action configs of a
# CcToolchainConfigInfo (structs made by rules_cc's cc_toolchain_config_lib),
# selects the ones a request enables, and expands their flags and environment
# against build variables.
#
# Starlark has no recursion and no `while`, so nesting is a stack and loops are
# bounded by what they walk.

_MAX = 1000000

def _by_name(config):
    """The selectables of a config by name, in the order they were defined."""
    selectables = {}
    order = []
    for f in config._features_DO_NOT_USE:
        selectables[f.name] = struct(kind = "feature", name = f.name, item = f)
        order.append(f.name)
    action_configs = {}
    for a in config._action_configs_DO_NOT_USE:
        selectables[a.action_name] = struct(kind = "action_config", name = a.action_name, item = a)
        action_configs[a.action_name] = a
        order.append(a.action_name)
    implied_by = {name: [] for name in order}
    for name in order:
        for implied in selectables[name].item.implies:
            if implied not in selectables:
                fail("Invalid toolchain configuration: Feature or action config '%s', implied by '%s', is not defined" % (implied, name))
            implied_by[implied].append(name)
    return struct(selectables = selectables, order = order, implied_by = implied_by, action_configs = action_configs)

def _requirements_met(item, enabled):
    if not getattr(item, "requires", None):
        return True
    for feature_set in item.requires:
        met = True
        for name in feature_set.features:
            if name not in enabled:
                met = False
                break
        if met:
            return True
    return False

def _select(model, requested):
    """Which selectables a request enables (Bazel's FeatureSelection)."""
    requested_set = {name: True for name in requested}
    enabled = {}
    for name in requested:
        if name not in model.selectables:
            continue
        stack = [name]
        for _ in range(_MAX):
            if not stack:
                break
            current = stack.pop()
            if current in enabled:
                continue
            enabled[current] = True
            stack.extend(model.selectables[current].item.implies)

    # Whatever is not requested, or implied by something enabled, or whose
    # implications and requirements fail, goes; until nothing does.
    for _ in range(len(model.order) + 1):
        removed = False
        for name in list(enabled.keys()):
            item = model.selectables[name].item
            wanted = name in requested_set
            if not wanted:
                for implier in model.implied_by[name]:
                    if implier in enabled:
                        wanted = True
                        break
            ok = wanted and _requirements_met(item, enabled)
            if ok:
                for implied in item.implies:
                    if implied not in enabled:
                        ok = False
                        break
            if not ok:
                enabled.pop(name)
                removed = True
        if not removed:
            break

    provided = {}
    for name in model.order:
        if name in enabled and model.selectables[name].kind == "feature":
            for symbol in getattr(model.selectables[name].item, "provides", []):
                provided.setdefault(symbol, []).append(name)
    for symbol, names in provided.items():
        if len(names) > 1:
            fail("Symbol %s is provided by all of the following features: %s" % (symbol, " ".join(names)))
    return enabled

def _with_features_hold(with_features, enabled):
    """Any of the feature sets has all its features enabled and none of its not_features."""
    if not with_features:
        return True
    for wf in with_features:
        holds = True
        for name in wf.features:
            if name not in enabled:
                holds = False
                break
        if holds:
            for name in wf.not_features:
                if name in enabled:
                    holds = False
                    break
        if holds:
            return True
    return False

# ---- variables ---------------------------------------------------------------------

def cc_variables(values, parent = None):
    return struct(_cc_variables = True, _values = values, _parent = parent)

def combine_cc_variables(*variables):
    """The variables of each of `variables`; a later one's names replace an earlier one's."""
    merged = {}
    for v in variables:
        if v == None:
            continue
        if not hasattr(v, "_cc_variables"):
            fail("combine_cc_toolchain_variables: %s is not CcToolchainVariables" % type(v))
        for name, value in v._values.items():
            merged[name] = value
        if v._parent != None:
            fail("combine_cc_toolchain_variables: variables of an iteration cannot be combined")
    return cc_variables(merged)

_MISSING = struct(_missing = True)

def _field(value, name):
    if type(value) == "dict":
        return value.get(name, _MISSING)
    return getattr(value, name, _MISSING)

def _lookup(variables, path):
    """The value of the variable `path` ("a" or "a.b.c"), or _MISSING."""
    scope = variables
    for _ in range(256):
        if scope == None:
            return _MISSING
        if path in scope._values:
            return scope._values[path]
        first = path.split(".", 1)[0]
        if first != path and first in scope._values:
            value = scope._values[first]
            for part in path.split(".")[1:]:
                value = _field(value, part)
                if value == _MISSING:
                    return _MISSING
            return value
        scope = scope._parent
    return _MISSING

def _is_sequence(value):
    return type(value) in ("list", "tuple", "depset")

def _elements(value, name):
    t = type(value)
    if t == "depset":
        return value.to_list()
    if t in ("list", "tuple"):
        return value
    fail("Invalid toolchain configuration: Cannot expand variable '%s': expected sequence, found %s" % (name, t))

def _as_string(value, name):
    t = type(value)
    if t == "string":
        return value
    if t == "int":
        return str(value)
    if t == "bool":
        return "1" if value else "0"
    if t == "File":
        return value.path
    fail("Invalid toolchain configuration: Cannot expand variable '%s': expected string, found %s" % (name, t))

def _truthy(value):
    t = type(value)
    if t == "bool":
        return value
    if t == "int":
        return value != 0
    if t == "string":
        return value != ""
    if _is_sequence(value):
        return len(_elements(value, "")) > 0
    return True

def _expand_string(text, variables):
    if "%" not in text:
        return text
    out = []
    i = 0
    for _ in range(len(text) + 1):
        j = text.find("%", i)
        if j < 0:
            out.append(text[i:])
            break
        out.append(text[i:j])
        nxt = text[j + 1:j + 2]
        if nxt == "%":
            out.append("%")
            i = j + 2
        elif nxt == "{":
            k = text.find("}", j + 2)
            if k < 0:
                fail("Invalid toolchain configuration: unterminated variable reference in '%s'" % text)
            name = text[j + 2:k]
            value = _lookup(variables, name)
            if value == _MISSING:
                fail("Invalid toolchain configuration: Cannot find variable named '%s'." % name)
            out.append(_as_string(value, name))
            i = k + 1
        else:
            fail("Invalid toolchain configuration: expected '%%{' or '%%%%' in '%s'" % text)
    return "".join(out)

def _can_expand(group, variables):
    available = getattr(group, "expand_if_available", None)
    if available and _lookup(variables, available) == _MISSING:
        return False
    not_available = getattr(group, "expand_if_not_available", None)
    if not_available and _lookup(variables, not_available) != _MISSING:
        return False
    if group.expand_if_true:
        value = _lookup(variables, group.expand_if_true)
        if value == _MISSING or not _truthy(value):
            return False
    if group.expand_if_false:
        value = _lookup(variables, group.expand_if_false)
        if value == _MISSING or _truthy(value):
            return False
    if group.expand_if_equal:
        value = _lookup(variables, group.expand_if_equal.name)
        if value == _MISSING or _as_string(value, group.expand_if_equal.name) != group.expand_if_equal.value:
            return False
    return True

def _expand_groups(groups, variables):
    """The flags of `groups` (flag_group structs) against `variables`."""
    out = []
    stack = [(g, variables) for g in reversed(groups)]
    for _ in range(_MAX):
        if not stack:
            break
        group, scope = stack.pop()
        if not _can_expand(group, scope):
            continue
        scopes = [scope]
        if group.iterate_over:
            value = _lookup(scope, group.iterate_over)
            if value == _MISSING:
                fail("Invalid toolchain configuration: Cannot expand variable '%s' (iterate_over)" % group.iterate_over)
            scopes = [cc_variables({group.iterate_over: e}, scope) for e in _elements(value, group.iterate_over)]
        if group.flags:
            for s in scopes:
                for flag in group.flags:
                    out.append(_expand_string(flag, s))
        else:
            for s in reversed(scopes):
                for nested in reversed(group.flag_groups):
                    stack.append((nested, s))
    return out

def _expand_flag_sets(flag_sets, action_name, enabled, variables):
    out = []
    for flag_set in flag_sets:
        if action_name not in flag_set.actions:
            continue
        if not _with_features_hold(flag_set.with_features, enabled):
            continue
        # A flag set can require variables to be available.
        required = getattr(flag_set, "expand_if_all_available", None)
        if required:
            skip = False
            for name in required:
                if _lookup(variables, name) == _MISSING:
                    skip = True
                    break
            if skip:
                continue
        out.extend(_expand_groups(flag_set.flag_groups, variables))
    return out

# ---- feature configurations -----------------------------------------------------------

def _configure(model, tools_directory, requested):
    enabled = _select(model, requested)
    in_order = [model.selectables[name] for name in model.order if name in enabled]
    return struct(
        _model = model,
        _enabled = enabled,
        _features = [s.item for s in in_order if s.kind == "feature"],
        _action_configs = {s.name: s.item for s in in_order if s.kind == "action_config"},
        _requested = {name: True for name in requested},
        _tools_directory = tools_directory,
        is_enabled = lambda name: name in enabled,
        is_requested = lambda name: name in requested,
    )

def cc_toolchain_features(toolchain_config_info, tools_directory):
    model = _by_name(toolchain_config_info)
    defaults = [name for name in model.order if model.selectables[name].item.enabled]
    return struct(
        _model = model,
        _tools_directory = tools_directory,
        _artifact_name_patterns = toolchain_config_info._artifact_name_patterns_DO_NOT_USE,
        default_features_and_action_configs = lambda: defaults,
        configure_features = lambda requested_features: _configure(model, tools_directory, requested_features),
    )

def action_is_enabled(feature_configuration, action_name):
    return action_name in feature_configuration._action_configs

def get_tool_for_action(feature_configuration, action_name):
    tool = _tool_for_action(feature_configuration, action_name)
    path = tool.tool.path if tool.tool else tool.path
    if path.startswith("/") or not feature_configuration._tools_directory:
        return path
    return feature_configuration._tools_directory + "/" + path

def get_execution_requirements(feature_configuration, action_name):
    return list(_tool_for_action(feature_configuration, action_name).execution_requirements)

def _tool_for_action(feature_configuration, action_name):
    config = feature_configuration._action_configs.get(action_name)
    if config == None:
        fail("Action '%s' is not configured." % action_name)
    for tool in config.tools:
        if _with_features_hold(tool.with_features, feature_configuration._enabled):
            return tool
    fail("Matching tool for action %s not found for given feature configuration" % action_name)

def get_memory_inefficient_command_line(feature_configuration, action_name, variables):
    enabled = feature_configuration._enabled
    out = []
    config = feature_configuration._action_configs.get(action_name)
    if config != None:
        out.extend(_expand_flag_sets(config.flag_sets, action_name, enabled, variables))
    for feature in feature_configuration._features:
        out.extend(_expand_flag_sets(feature.flag_sets, action_name, enabled, variables))
    return out

def get_environment_variables(feature_configuration, action_name, variables):
    enabled = feature_configuration._enabled
    env = {}
    for feature in feature_configuration._features:
        for env_set in feature.env_sets:
            if action_name not in env_set.actions:
                continue
            if not _with_features_hold(env_set.with_features, enabled):
                continue
            for entry in env_set.env_entries:
                available = getattr(entry, "expand_if_available", None)
                if available and _lookup(variables, available) == _MISSING:
                    continue
                env[entry.key] = _expand_string(entry.value, variables)
    return env
