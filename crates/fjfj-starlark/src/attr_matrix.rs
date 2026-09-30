//! The `attr.*` matrix (buildfiji-mum.3.3): every keyword each builder takes,
//! against sixteen values of every type, as Bazel 9.2.0 answered them. Kept
//! apart from `attr_tests.rs` because it is most of the data.

pub(crate) const ATTR_MATRIX: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(attr.bool(default=1))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=[]))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default={"a":"b"}))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=lambda: 1))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'function', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(doc=1))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=[]))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc={"a":"b"}))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=lambda: 1))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(mandatory=1))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=[]))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory={"a":"b"}))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=lambda: 1))"#,
        Err(
            r#"in call to bool(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=1))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=[]))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable={"a":"b"}))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=lambda: 1))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (r#"print(attr.int(default=1))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(default=[]))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'list', want 'int'"#),
    ),
    (
        r#"print(attr.int(default={"a":"b"}))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'dict', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=lambda: 1))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'function', want 'int'"#),
    ),
    (
        r#"print(attr.int(doc=1))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=[]))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc={"a":"b"}))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=lambda: 1))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(mandatory=1))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=[]))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory={"a":"b"}))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=lambda: 1))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'function', want 'bool'"#),
    ),
    (
        r#"print(attr.int(values=1))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'int', want 'sequence'"#),
    ),
    (r#"print(attr.int(values=[]))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(values={"a":"b"}))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.int(values=lambda: 1))"#,
        Err(
            r#"in call to int(), parameter 'values' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=1))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=[]))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable={"a":"b"}))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=lambda: 1))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=1))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=[]))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.int_list(default={"a":"b"}))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=lambda: 1))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=1))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=[]))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc={"a":"b"}))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=lambda: 1))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=1))"#,
        Err(r#"in call to int_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int_list(mandatory=[]))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=lambda: 1))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=1))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=[]))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable={"a":"b"}))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=lambda: 1))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=1))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=[]))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=[]))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'list', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default={"a":"b"}))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'dict', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=lambda: 1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(doc=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=[]))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc={"a":"b"}))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(mandatory=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=[]))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory={"a":"b"}))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=[]))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable={"a":"b"}))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=[]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files={"a":"b"}))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'function', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_single_file=1))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file={"a":"b"}))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_single_file=lambda: 1))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_rules=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=[]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules={"a":"b"}))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'function', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=1))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (r#"print(attr.label(providers=[]))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(providers={"a":"b"}))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(flags=1))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'int', want 'sequence'"#),
    ),
    (r#"print(attr.label(flags=[]))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(flags={"a":"b"}))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'flags' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(cfg=1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=[]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg={"a":"b"}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=lambda: 1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(aspects=1))"#,
        Err(r#"in call to label(), parameter 'aspects' got value of type 'int', want 'sequence'"#),
    ),
    (r#"print(attr.label(aspects=[]))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(aspects={"a":"b"}))"#,
        Err(r#"in call to label(), parameter 'aspects' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(aspects=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(executable=1))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable=[]))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable={"a":"b"}))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'executable' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=1))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=[]))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations={"a":"b"}))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=lambda: 1))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=[]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution={"a":"b"}))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=lambda: 1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=[]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'list', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"a":"b"}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=lambda: 1))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=[]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=[]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=[]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=[]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=[]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'function', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=[]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'function', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=[]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=[]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=[]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg={"a":"b"}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=lambda: 1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=[]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=[]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations={"a":"b"}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=lambda: 1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=1))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=[]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution={"a":"b"}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=lambda: 1))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_list(default=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=[]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'dict', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=lambda: 1))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(doc=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=[]))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=[]))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=[]))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=[]))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=[]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_files={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'function', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=[]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_rules={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'function', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=1))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=[]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(providers={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=[]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(flags={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=[]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg={"a":"b"}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=lambda: 1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=[]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(aspects={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=[]))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations={"a":"b"}))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=lambda: 1))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=1))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=[]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution={"a":"b"}))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=lambda: 1))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list_dict(default=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=[]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":"b"}))"#,
        Err(r#"expected value of type 'list(label)' for dict value element, but got "b" (string)"#),
    ),
    (
        r#"print(attr.label_list_dict(default=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'function', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=[]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=[]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=[]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=[]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=[]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_files={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'function', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=[]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'function', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=[]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(providers={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=[]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(flags={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=[]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg={"a":"b"}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=lambda: 1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=[]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(aspects={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=[]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations={"a":"b"}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=lambda: 1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=1))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=[]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution={"a":"b"}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=lambda: 1))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.output(doc=1))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=[]))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc={"a":"b"}))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=lambda: 1))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(mandatory=1))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.output(mandatory=[]))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.output(mandatory={"a":"b"}))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.output(mandatory=lambda: 1))"#,
        Err(
            r#"in call to output(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=1))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=[]))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc={"a":"b"}))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=lambda: 1))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=1))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=[]))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=lambda: 1))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=1))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=[]))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(default=1))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=[]))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'list', want 'string'"#),
    ),
    (
        r#"print(attr.string(default={"a":"b"}))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'dict', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=lambda: 1))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'function', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=1))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=[]))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc={"a":"b"}))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=lambda: 1))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory=1))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=[]))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory={"a":"b"}))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=lambda: 1))"#,
        Err(
            r#"in call to string(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(values=1))"#,
        Err(r#"in call to string(), parameter 'values' got value of type 'int', want 'sequence'"#),
    ),
    (r#"print(attr.string(values=[]))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(values={"a":"b"}))"#,
        Err(r#"in call to string(), parameter 'values' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.string(values=lambda: 1))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=1))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=[]))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable={"a":"b"}))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=lambda: 1))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=[]))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default={"a":"b"}))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(default=lambda: 1))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'function', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=[]))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc={"a":"b"}))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=lambda: 1))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=[]))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=lambda: 1))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=[]))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable={"a":"b"}))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=lambda: 1))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=[]))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=[]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'list', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"a":"b"}))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=lambda: 1))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=[]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=[]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=[]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=[]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=[]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'function', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=[]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'function', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=[]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=[]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=[]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg={"a":"b"}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=lambda: 1))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=[]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects={"a":"b"}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=lambda: 1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=1))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=[]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution={"a":"b"}))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=lambda: 1))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_list(default=1))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=[]))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(default={"a":"b"}))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=lambda: 1))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=1))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=[]))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc={"a":"b"}))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=lambda: 1))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=1))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=[]))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=lambda: 1))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=1))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=[]))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable={"a":"b"}))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=lambda: 1))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=1))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=[]))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=[]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":"b"}))"#,
        Err(
            r#"expected value of type 'list(string)' for dict value element, but got "b" (string)"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=lambda: 1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'function', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=[]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc={"a":"b"}))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=lambda: 1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'function', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=[]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory={"a":"b"}))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=lambda: 1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=[]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable={"a":"b"}))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=lambda: 1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'function', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=[]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty={"a":"b"}))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=lambda: 1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.bool(default="a"))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=["a"]))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=depset()))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'depset', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=print))"#,
        Err(
            r#"in call to bool(), parameter 'default' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (r#"print(attr.bool(doc="a"))"#, Ok(r#"<attr.bool>"#)),
    (
        r#"print(attr.bool(doc=["a"]))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=depset()))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=print))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(mandatory="a"))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=["a"]))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=depset()))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'depset', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=print))"#,
        Err(
            r#"in call to bool(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable="a"))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=["a"]))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=depset()))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=print))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(default="a"))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'string', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=["a"]))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'list', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=depset()))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'depset', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=print))"#,
        Err(
            r#"in call to int(), parameter 'default' got value of type 'builtin_function_or_method', want 'int'"#,
        ),
    ),
    (r#"print(attr.int(doc="a"))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(doc=["a"]))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=depset()))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=print))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(mandatory="a"))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=["a"]))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=depset()))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'depset', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=print))"#,
        Err(
            r#"in call to int(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int(values="a"))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'string', want 'sequence'"#),
    ),
    (r#"print(attr.int(values=["a"]))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(values=depset()))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'depset', want 'sequence'"#),
    ),
    (
        r#"print(attr.int(values=print))"#,
        Err(
            r#"in call to int(), parameter 'values' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable="a"))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=["a"]))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=depset()))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=print))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default="a"))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=["a"]))"#,
        Err(
            r#"expected value of type 'int' for element 0 of parameter 'default' of attribute '', but got "a" (string)"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=depset()))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=print))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (r#"print(attr.int_list(doc="a"))"#, Ok(r#"<attr.int_list>"#)),
    (
        r#"print(attr.int_list(doc=["a"]))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=depset()))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=print))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory="a"))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=["a"]))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=depset()))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=print))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable="a"))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=["a"]))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=depset()))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=print))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty="a"))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=["a"]))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=depset()))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=print))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (r#"print(attr.label(default="a"))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(default=["a"]))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'list', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=depset()))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'depset', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=print))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'builtin_function_or_method', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (r#"print(attr.label(doc="a"))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(doc=["a"]))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=depset()))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=print))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(mandatory="a"))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=["a"]))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=depset()))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'depset', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=print))"#,
        Err(
            r#"in call to label(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable="a"))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=["a"]))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=depset()))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=print))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files="a"))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'string', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=["a"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=depset()))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'depset', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=print))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'builtin_function_or_method', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_single_file="a"))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_single_file=["a"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file=depset()))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_single_file=print))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_rules="a"))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'string', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=["a"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=depset()))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'depset', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=print))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'builtin_function_or_method', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(providers="a"))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=["a"]))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=depset()))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=print))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(flags="a"))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'string', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(flags=depset()))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'depset', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=print))"#,
        Err(
            r#"in call to label(), parameter 'flags' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(cfg="a"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=["a"]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=depset()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=print))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(aspects="a"))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(aspects=["a"]))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.label(aspects=depset()))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(aspects=print))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(executable="a"))"#,
        Err(
            r#"in call to label(), parameter 'executable' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(executable=["a"]))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable=depset()))"#,
        Err(
            r#"in call to label(), parameter 'executable' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(executable=print))"#,
        Err(
            r#"in call to label(), parameter 'executable' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations="a"))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=["a"]))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=depset()))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=print))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(for_dependency_resolution="a"))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=["a"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=depset()))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=print))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'string', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=["a"]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'list', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'depset', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'builtin_function_or_method', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc="a"))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=["a"]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=["a"]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=["a"]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=["a"]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'string', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=["a"]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'depset', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'builtin_function_or_method', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'string', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=["a"]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'depset', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'builtin_function_or_method', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=["a"]))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg="a"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=["a"]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=depset()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=print))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=["a"]))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations="a"))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=["a"]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=depset()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=print))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution="a"))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=["a"]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=depset()))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=print))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_list(default="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'string', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=["a"]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'depset', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=print))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'builtin_function_or_method', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc="a"))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(doc=["a"]))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=print))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=["a"]))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=print))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=["a"]))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=print))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=["a"]))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=print))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'string', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=["a"]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_files=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'depset', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=print))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'builtin_function_or_method', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'string', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=["a"]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_rules=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'depset', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=print))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'builtin_function_or_method', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=["a"]))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.label_list(providers=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=print))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label_list(flags=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=print))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg="a"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=["a"]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=depset()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=print))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=["a"]))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.label_list(aspects=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=print))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations="a"))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=["a"]))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=print))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution="a"))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=["a"]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=depset()))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=print))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list_dict(default="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'string', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=["a"]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'depset', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'builtin_function_or_method', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc="a"))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(doc=["a"]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=["a"]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=["a"]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=["a"]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'string', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=["a"]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'depset', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'builtin_function_or_method', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'string', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=["a"]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'depset', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'builtin_function_or_method', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=["a"]))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.label_list_dict(providers=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label_list_dict(flags=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg="a"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=["a"]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=depset()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=print))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=["a"]))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.label_list_dict(aspects=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations="a"))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=["a"]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=depset()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=print))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution="a"))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=["a"]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=depset()))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=print))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (r#"print(attr.output(doc="a"))"#, Ok(r#"<attr.output>"#)),
    (
        r#"print(attr.output(doc=["a"]))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=depset()))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=print))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(mandatory="a"))"#,
        Err(
            r#"in call to output(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output(mandatory=["a"]))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.output(mandatory=depset()))"#,
        Err(
            r#"in call to output(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output(mandatory=print))"#,
        Err(
            r#"in call to output(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc="a"))"#,
        Ok(r#"<attr.output_list>"#),
    ),
    (
        r#"print(attr.output_list(doc=["a"]))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=depset()))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=print))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory="a"))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=["a"]))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=depset()))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=print))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty="a"))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=["a"]))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=depset()))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=print))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (r#"print(attr.string(default="a"))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(default=["a"]))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'list', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=depset()))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'depset', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(default=print))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'builtin_function_or_method', want 'string'"#,
        ),
    ),
    (r#"print(attr.string(doc="a"))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(doc=["a"]))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=depset()))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=print))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory="a"))"#,
        Err(
            r#"in call to string(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory=["a"]))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=depset()))"#,
        Err(
            r#"in call to string(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory=print))"#,
        Err(
            r#"in call to string(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(values="a"))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(values=["a"]))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(values=depset()))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(values=print))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable="a"))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=["a"]))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=depset()))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=print))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default="a"))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'string', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=["a"]))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=depset()))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'depset', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=print))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'builtin_function_or_method', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc="a"))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(doc=["a"]))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=depset()))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=print))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory="a"))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=["a"]))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=depset()))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=print))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable="a"))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=["a"]))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=depset()))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=print))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty="a"))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=["a"]))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=depset()))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=print))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'string', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=["a"]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'list', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'depset', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'builtin_function_or_method', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc="a"))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=["a"]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=["a"]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=["a"]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=["a"]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'string', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=["a"]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'depset', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'builtin_function_or_method', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'string', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=["a"]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'depset', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'builtin_function_or_method', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=["a"]))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg="a"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=["a"]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=depset()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=print))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects="a"))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=["a"]))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=depset()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=print))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution="a"))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=["a"]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=depset()))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=print))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_list(default="a"))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=["a"]))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(default=depset()))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=print))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'builtin_function_or_method', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc="a"))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(doc=["a"]))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=depset()))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=print))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory="a"))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=["a"]))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=depset()))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=print))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable="a"))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=["a"]))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=depset()))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=print))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty="a"))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=["a"]))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=depset()))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=print))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default="a"))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'string', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=["a"]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=depset()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'depset', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=print))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'builtin_function_or_method', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc="a"))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(doc=["a"]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=depset()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'depset', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=print))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'builtin_function_or_method', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory="a"))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=["a"]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=depset()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=print))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable="a"))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'string', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=["a"]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=depset()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'depset', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=print))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'builtin_function_or_method', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty="a"))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=["a"]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=depset()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'depset', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=print))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'builtin_function_or_method', want 'bool'"#,
        ),
    ),
    (r#"print(attr.bool(default=True))"#, Ok(r#"<attr.bool>"#)),
    (
        r#"print(attr.bool(default=[1]))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=Label("//a:b")))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'Label', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=("a",)))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(doc=True))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=[1]))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=Label("//a:b")))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=("a",)))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (r#"print(attr.bool(mandatory=True))"#, Ok(r#"<attr.bool>"#)),
    (
        r#"print(attr.bool(mandatory=[1]))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=Label("//a:b")))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'Label', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=("a",)))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(configurable=True))"#,
        Ok(r#"<attr.bool>"#),
    ),
    (
        r#"print(attr.bool(configurable=[1]))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=("a",)))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(default=True))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'bool', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=[1]))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'list', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=Label("//a:b")))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'Label', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=("a",)))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'tuple', want 'int'"#),
    ),
    (
        r#"print(attr.int(doc=True))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=[1]))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=Label("//a:b")))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=("a",)))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (r#"print(attr.int(mandatory=True))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(mandatory=[1]))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=Label("//a:b")))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'Label', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=("a",)))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(attr.int(values=True))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'bool', want 'sequence'"#),
    ),
    (r#"print(attr.int(values=[1]))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(values=Label("//a:b")))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'Label', want 'sequence'"#),
    ),
    (r#"print(attr.int(values=("a",)))"#, Ok(r#"<attr.int>"#)),
    (r#"print(attr.int(configurable=True))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(configurable=[1]))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=("a",)))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=True))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=[1]))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.int_list(default=Label("//a:b")))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=("a",)))"#,
        Err(
            r#"expected value of type 'int' for element 0 of parameter 'default' of attribute '', but got "a" (string)"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=True))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=[1]))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=Label("//a:b")))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=("a",)))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=True))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.int_list(mandatory=[1]))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=("a",)))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=True))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.int_list(configurable=[1]))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=("a",)))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=True))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.int_list(allow_empty=[1]))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=("a",)))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(default=True))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'bool', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=[1]))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'list', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=Label("//a:b")))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default=("a",)))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'tuple', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=True))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=[1]))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=Label("//a:b")))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=("a",)))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(mandatory=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(mandatory=[1]))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=Label("//a:b")))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'Label', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=("a",)))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(attr.label(configurable=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(configurable=[1]))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=("a",)))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_files=Label("//a:b")))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'Label', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=("a",)))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_single_file=Label("//a:b")))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_single_file=("a",)))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=True))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'bool', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=Label("//a:b")))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'Label', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=("a",)))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=True))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=[1]))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=Label("//a:b")))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=("a",)))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.label(flags=True))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'bool', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=[1]))"#,
        Err(r#"at index 0 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(flags=Label("//a:b")))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'Label', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=("a",)))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(cfg=True))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=[1]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=Label("//a:b")))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=("a",)))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(aspects=True))"#,
        Err(r#"in call to label(), parameter 'aspects' got value of type 'bool', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(aspects=[1]))"#,
        Err(r#"at index 0 of aspects, got element of type int, want Aspect"#),
    ),
    (
        r#"print(attr.label(aspects=Label("//a:b")))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(aspects=("a",)))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.label(executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(executable=[1]))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable=Label("//a:b")))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'Label', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable=("a",)))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(attr.label(skip_validations=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(skip_validations=[1]))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=Label("//a:b")))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=("a",)))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=[1]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=Label("//a:b")))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=("a",)))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=True))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'bool', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=[1]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'list', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'Label', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=("a",)))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'tuple', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=True))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=[1]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=("a",)))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=True))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=[1]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=("a",)))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=True))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=[1]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=("a",)))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=True))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=[1]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=("a",)))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=True))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'Label', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=("a",)))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=True))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'bool', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'Label', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=("a",)))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=True))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=[1]))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=("a",)))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=True))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=[1]))"#,
        Err(r#"at index 0 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=("a",)))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=True))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=[1]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=Label("//a:b")))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=("a",)))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=True))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=[1]))"#,
        Err(r#"at index 0 of aspects, got element of type int, want Aspect"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=("a",)))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=True))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=[1]))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=Label("//a:b")))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=("a",)))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=True))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=[1]))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=Label("//a:b")))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=("a",)))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_list(default=True))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'bool', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=[1]))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute 'label_list', but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'Label', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=("a",)))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(doc=True))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=[1]))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=("a",)))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=True))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(mandatory=[1]))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=("a",)))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=True))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(configurable=[1]))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=("a",)))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=True))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_empty=[1]))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=("a",)))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=True))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label_list(allow_files=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'Label', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=("a",)))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_rules=True))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'bool', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'Label', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=("a",)))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(providers=True))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=[1]))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label_list(providers=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=("a",)))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.label_list(flags=True))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=[1]))"#,
        Err(r#"at index 0 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label_list(flags=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=("a",)))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label_list(cfg=True))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=[1]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=Label("//a:b")))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=("a",)))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=True))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=[1]))"#,
        Err(r#"at index 0 of aspects, got element of type int, want Aspect"#),
    ),
    (
        r#"print(attr.label_list(aspects=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=("a",)))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.label_list(skip_validations=True))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(skip_validations=[1]))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=Label("//a:b")))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=("a",)))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=True))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=[1]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=Label("//a:b")))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=("a",)))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list_dict(default=True))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'bool', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=[1]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'Label', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=("a",)))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'tuple', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=True))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=[1]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=("a",)))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=True))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=[1]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=("a",)))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=True))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(configurable=[1]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=("a",)))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=True))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=[1]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=("a",)))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=True))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'Label', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=("a",)))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=True))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'bool', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'Label', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=("a",)))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(providers=True))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=[1]))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label_list_dict(providers=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=("a",)))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.label_list_dict(flags=True))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=[1]))"#,
        Err(r#"at index 0 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label_list_dict(flags=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=("a",)))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label_list_dict(cfg=True))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=[1]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=Label("//a:b")))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=("a",)))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=True))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=[1]))"#,
        Err(r#"at index 0 of aspects, got element of type int, want Aspect"#),
    ),
    (
        r#"print(attr.label_list_dict(aspects=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=("a",)))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=True))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=[1]))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=Label("//a:b")))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=("a",)))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=True))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=[1]))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=Label("//a:b")))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=("a",)))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.output(doc=True))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=[1]))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=Label("//a:b")))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=("a",)))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(mandatory=True))"#,
        Ok(r#"<attr.output>"#),
    ),
    (
        r#"print(attr.output(mandatory=[1]))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.output(mandatory=Label("//a:b")))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'Label', want 'bool'"#),
    ),
    (
        r#"print(attr.output(mandatory=("a",)))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(attr.output_list(doc=True))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=[1]))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=Label("//a:b")))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=("a",)))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=True))"#,
        Ok(r#"<attr.output_list>"#),
    ),
    (
        r#"print(attr.output_list(mandatory=[1]))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=("a",)))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=True))"#,
        Ok(r#"<attr.output_list>"#),
    ),
    (
        r#"print(attr.output_list(allow_empty=[1]))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=("a",)))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(default=True))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'bool', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=[1]))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'list', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=Label("//a:b")))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'Label', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=("a",)))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'tuple', want 'string'"#),
    ),
    (
        r#"print(attr.string(doc=True))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=[1]))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=Label("//a:b")))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=("a",)))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory=True))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(mandatory=[1]))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=Label("//a:b")))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'Label', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=("a",)))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#),
    ),
    (
        r#"print(attr.string(values=True))"#,
        Err(r#"in call to string(), parameter 'values' got value of type 'bool', want 'sequence'"#),
    ),
    (r#"print(attr.string(values=[1]))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(values=Label("//a:b")))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(values=("a",)))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(configurable=True))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(configurable=[1]))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=("a",)))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=True))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'bool', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=[1]))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=Label("//a:b")))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'Label', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=("a",)))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'tuple', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=True))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=[1]))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=Label("//a:b")))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=("a",)))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=True))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(mandatory=[1]))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=("a",)))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=True))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(configurable=[1]))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=("a",)))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=True))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(allow_empty=[1]))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=("a",)))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=True))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'bool', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=[1]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'list', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'Label', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=("a",)))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'tuple', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=True))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=[1]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=("a",)))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=True))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=[1]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=("a",)))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=True))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=[1]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=("a",)))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=True))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=[1]))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=("a",)))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=True))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'Label', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=("a",)))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=True))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'bool', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'Label', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=("a",)))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=True))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=[1]))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=("a",)))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=True))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=[1]))"#,
        Err(r#"at index 0 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=("a",)))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=True))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=[1]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=Label("//a:b")))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=("a",)))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=True))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=[1]))"#,
        Err(r#"at index 0 of aspects, got element of type int, want Aspect"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=Label("//a:b")))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=("a",)))"#,
        Err(r#"at index 0 of aspects, got element of type string, want Aspect"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=True))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=[1]))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=Label("//a:b")))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=("a",)))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_list(default=True))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'bool', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=[1]))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute '', but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=Label("//a:b")))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'Label', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=("a",)))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(doc=True))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=[1]))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=Label("//a:b")))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=("a",)))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=True))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(mandatory=[1]))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=("a",)))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=True))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(configurable=[1]))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=("a",)))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=True))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(allow_empty=[1]))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=("a",)))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=True))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'bool', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=[1]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=Label("//a:b")))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'Label', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=("a",)))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'tuple', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=True))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=[1]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=Label("//a:b")))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'Label', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=("a",)))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'tuple', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=True))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=[1]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=Label("//a:b")))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=("a",)))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=True))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(configurable=[1]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'list', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=Label("//a:b")))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'Label', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=("a",)))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'tuple', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=True))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=[1]))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=Label("//a:b")))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'Label', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=("a",)))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'tuple', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.bool(default=None))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'NoneType', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default={}))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=1.5))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'float', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=attr.string()))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'Attribute', want 'bool'"#),
    ),
    (r#"print(attr.bool(doc=None))"#, Ok(r#"<attr.bool>"#)),
    (
        r#"print(attr.bool(doc={}))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=1.5))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=attr.string()))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(mandatory=None))"#,
        Err(
            r#"in call to bool(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.bool(mandatory={}))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=1.5))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'float', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(mandatory=attr.string()))"#,
        Err(
            r#"in call to bool(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=None))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable={}))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=1.5))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(configurable=attr.string()))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(default=None))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'NoneType', want 'int'"#),
    ),
    (
        r#"print(attr.int(default={}))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'dict', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=1.5))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'float', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=attr.string()))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'Attribute', want 'int'"#),
    ),
    (r#"print(attr.int(doc=None))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(doc={}))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=1.5))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=attr.string()))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(mandatory=None))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory={}))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=1.5))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'float', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=attr.string()))"#,
        Err(
            r#"in call to int(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int(values=None))"#,
        Err(
            r#"in call to int(), parameter 'values' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int(values={}))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.int(values=1.5))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'float', want 'sequence'"#),
    ),
    (
        r#"print(attr.int(values=attr.string()))"#,
        Err(
            r#"in call to int(), parameter 'values' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=None))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable={}))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=1.5))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=attr.string()))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=None))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default={}))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=1.5))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=attr.string()))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=None))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.int_list(doc={}))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=1.5))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=attr.string()))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=None))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory={}))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=1.5))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=attr.string()))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=None))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable={}))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=1.5))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(configurable=attr.string()))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=None))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty={}))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=1.5))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=attr.string()))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (r#"print(attr.label(default=None))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(default={}))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'dict', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1.5))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'float', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'Attribute', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (r#"print(attr.label(doc=None))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(doc={}))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1.5))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(mandatory=None))"#,
        Err(
            r#"in call to label(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(mandatory={}))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=1.5))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'float', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=None))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable={}))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=1.5))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files={}))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1.5))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'float', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'Attribute', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_single_file=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file={}))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_single_file=1.5))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_single_file=attr.string()))"#,
        Err(r#"allow_single_file should be a boolean or a string list"#),
    ),
    (
        r#"print(attr.label(allow_rules=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules={}))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=1.5))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'float', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'Attribute', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=None))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers={}))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=1.5))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(flags=None))"#,
        Err(
            r#"in call to label(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(flags={}))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=1.5))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'float', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'flags' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (r#"print(attr.label(cfg=None))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(cfg={}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=1.5))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg=attr.string()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(aspects=None))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(aspects={}))"#,
        Err(r#"in call to label(), parameter 'aspects' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(aspects=1.5))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(aspects=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(executable=None))"#,
        Err(
            r#"in call to label(), parameter 'executable' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(executable={}))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable=1.5))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'float', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'executable' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=None))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations={}))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=1.5))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=attr.string()))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution={}))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=1.5))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=attr.string()))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'NoneType', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'float', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'Attribute', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'float', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'Attribute', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'float', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'Attribute', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg={}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=1.5))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=attr.string()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations={}))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=1.5))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=attr.string()))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution={}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=1.5))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=attr.string()))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_list(default=None))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'NoneType', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default={}))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'dict', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'float', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'Attribute', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(doc={}))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=None))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory={}))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=None))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable={}))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=None))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty={}))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_files={}))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'float', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'Attribute', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_rules={}))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'float', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'Attribute', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=None))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers={}))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=None))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags={}))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(cfg={}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=1.5))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=attr.string()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=None))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects={}))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=None))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations={}))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=1.5))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(skip_validations=attr.string()))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution={}))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=1.5))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=attr.string()))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list_dict(default=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default={}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(default=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'float', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'Attribute', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(doc={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_files={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'float', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'Attribute', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'float', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'Attribute', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(cfg={}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=1.5))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=attr.string()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations={}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=1.5))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=attr.string()))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution={}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=1.5))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=attr.string()))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (r#"print(attr.output(doc=None))"#, Ok(r#"<attr.output>"#)),
    (
        r#"print(attr.output(doc={}))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=1.5))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=attr.string()))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output(mandatory=None))"#,
        Err(
            r#"in call to output(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output(mandatory={}))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.output(mandatory=1.5))"#,
        Err(r#"in call to output(), parameter 'mandatory' got value of type 'float', want 'bool'"#),
    ),
    (
        r#"print(attr.output(mandatory=attr.string()))"#,
        Err(
            r#"in call to output(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=None))"#,
        Ok(r#"<attr.output_list>"#),
    ),
    (
        r#"print(attr.output_list(doc={}))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=1.5))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=attr.string()))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=None))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory={}))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=1.5))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=attr.string()))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=None))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty={}))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=1.5))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_empty=attr.string()))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(default=None))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'NoneType', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(default={}))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'dict', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=1.5))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'float', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=attr.string()))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'Attribute', want 'string'"#,
        ),
    ),
    (r#"print(attr.string(doc=None))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(doc={}))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=1.5))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=attr.string()))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory=None))"#,
        Err(
            r#"in call to string(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory={}))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'dict', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=1.5))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'float', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=attr.string()))"#,
        Err(
            r#"in call to string(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(values=None))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(values={}))"#,
        Err(r#"in call to string(), parameter 'values' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.string(values=1.5))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(values=attr.string()))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=None))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable={}))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=1.5))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=attr.string()))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=None))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default={}))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(default=1.5))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'float', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=attr.string()))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'Attribute', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=None))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(doc={}))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=1.5))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=attr.string()))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=None))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory={}))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=1.5))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=attr.string()))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=None))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable={}))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=1.5))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=attr.string()))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=None))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty={}))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=1.5))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=attr.string()))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'NoneType', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={}))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'float', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'Attribute', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'dict', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'float', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'Attribute', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'dict', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'float', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'Attribute', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg={}))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=1.5))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=attr.string()))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects={}))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=1.5))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=attr.string()))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution={}))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=1.5))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=attr.string()))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_list(default=None))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default={}))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=1.5))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'float', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=attr.string()))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'Attribute', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=None))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(doc={}))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=1.5))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=attr.string()))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=None))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory={}))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=1.5))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=attr.string()))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=None))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable={}))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=1.5))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=attr.string()))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=None))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty={}))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=1.5))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=attr.string()))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=None))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default={}))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(default=1.5))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'float', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=attr.string()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'Attribute', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=None))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(doc={}))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'dict', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=1.5))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'float', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=attr.string()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'Attribute', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=None))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory={}))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=1.5))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=attr.string()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=None))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable={}))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'dict', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=1.5))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'float', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=attr.string()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'Attribute', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=None))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty={}))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'dict', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=1.5))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'float', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=attr.string()))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'Attribute', want 'bool'"#,
        ),
    ),
];
