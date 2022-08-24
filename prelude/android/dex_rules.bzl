load("@fbcode//buck2/prelude/android:android_providers.bzl", "DexFilesInfo")
load("@fbcode//buck2/prelude/android:voltron.bzl", "get_apk_module_graph_info", "get_root_module_only_apk_module_graph_info", "is_root_module")
load("@fbcode//buck2/prelude/java:dex.bzl", "get_dex_produced_from_java_library")
load("@fbcode//buck2/prelude/java:dex_toolchain.bzl", "DexToolchainInfo")
load("@fbcode//buck2/prelude/java:java_library.bzl", "compile_to_jar")
load("@fbcode//buck2/prelude/utils:utils.bzl", "expect", "flatten")

_DEX_MERGE_OPTIONS = ["--no-desugar", "--no-optimize"]

SplitDexMergeConfig = record(
    dex_compression = str.type,
    primary_dex_patterns = [str.type],
    secondary_dex_weight_limit_bytes = int.type,
)

def _get_dex_compression(ctx: "context") -> str.type:
    is_exopackage_enabled_for_secondary_dexes = "secondary_dex" in ctx.attrs.exopackage_modes
    default_dex_compression = "jar" if is_exopackage_enabled_for_secondary_dexes else "raw"
    dex_compression = ctx.attrs.dex_compression or default_dex_compression
    expect(
        dex_compression in ["raw", "jar", "xz", "xzs"],
        "Only 'raw', 'jar', 'xz' and 'xzs' dex compression are supported at this time!",
    )

    return dex_compression

def get_split_dex_merge_config(
        ctx: "context",
        android_toolchain: "AndroidToolchainInfo") -> "SplitDexMergeConfig":
    return SplitDexMergeConfig(
        dex_compression = _get_dex_compression(ctx),
        primary_dex_patterns = ctx.attrs.primary_dex_patterns,
        secondary_dex_weight_limit_bytes = (
            ctx.attrs.secondary_dex_weight_limit or
            android_toolchain.secondary_dex_weight_limit
        ),
    )

def get_single_primary_dex(
        ctx: "context",
        android_toolchain: "AndroidToolchainInfo",
        java_library_jars: ["artifact"],
        is_optimized: bool.type) -> "DexFilesInfo":
    d8_cmd = cmd_args(android_toolchain.d8_command[RunInfo])

    output_dex_file = ctx.actions.declare_output("classes.dex")
    d8_cmd.add(["--output-dex-file", output_dex_file.as_output()])

    jar_to_dex_file = ctx.actions.write("jar_to_dex_file.txt", java_library_jars)
    d8_cmd.add(["--files-to-dex-list", jar_to_dex_file])
    d8_cmd.hidden(java_library_jars)

    d8_cmd.add(["--android-jar", android_toolchain.android_jar])
    if not is_optimized:
        d8_cmd.add("--no-optimize")

    ctx.actions.run(d8_cmd, category = "d8", identifier = "{}:{}".format(ctx.label.package, ctx.label.name))

    return DexFilesInfo(
        primary_dex = output_dex_file,
        secondary_dex_dirs = [],
        proguard_text_files_path = None,
    )

def get_multi_dex(
        ctx: "context",
        android_toolchain: "AndroidToolchainInfo",
        java_library_jars_to_owners: {"artifact": "target_label"},
        primary_dex_patterns: [str.type],
        proguard_configuration_output_file: ["artifact", None],
        proguard_mapping_output_file: ["artifact", None],
        is_optimized: bool.type,
        apk_module_graph_file: ["artifact", None] = None) -> "DexFilesInfo":
    primary_dex_file = ctx.actions.declare_output("classes.dex")
    root_module_secondary_dex_output_dir = ctx.actions.declare_output("root_module_secondary_dex_output_dir")
    secondary_dex_dir = ctx.actions.declare_output("secondary_dex_output_dir")

    # dynamic actions are not valid with no input, but it's easier to use the same code regardless,
    # so just create an empty input.
    inputs = [apk_module_graph_file] if apk_module_graph_file else [ctx.actions.write("empty_artifact_for_multi_dex_dynamic_action", [])]
    outputs = [primary_dex_file, root_module_secondary_dex_output_dir, secondary_dex_dir]

    def do_multi_dex(ctx: "context"):
        apk_module_graph_info = get_apk_module_graph_info(ctx, apk_module_graph_file) if apk_module_graph_file else get_root_module_only_apk_module_graph_info()
        target_to_module_mapping_function = apk_module_graph_info.target_to_module_mapping_function
        module_to_jars = {}
        for java_library_jar, owner in java_library_jars_to_owners.items():
            module = target_to_module_mapping_function(str(owner))
            module_to_jars.setdefault(module, []).append(java_library_jar)

        secondary_dex_dir_srcs = {}
        for module, jars in module_to_jars.items():
            multi_dex_cmd = cmd_args(android_toolchain.multi_dex_command[RunInfo])

            if is_root_module(module):
                multi_dex_cmd.add("--primary-dex", ctx.outputs[primary_dex_file].as_output())
                multi_dex_cmd.add("--primary-dex-patterns-path", ctx.actions.write("primary_dex_patterns", primary_dex_patterns))
                multi_dex_cmd.add("--secondary-dex-output-dir", ctx.outputs[root_module_secondary_dex_output_dir].as_output())
            else:
                secondary_dex_dir_for_module = ctx.actions.declare_output("secondary_dex_output_dir_for_module_{}".format(module))
                secondary_dex_subdir = secondary_dex_dir_for_module.project(_get_secondary_dex_subdir(module))
                secondary_dex_dir_srcs[_get_secondary_dex_subdir(module)] = secondary_dex_subdir
                multi_dex_cmd.add("--secondary-dex-output-dir", secondary_dex_dir_for_module.as_output())
                multi_dex_cmd.add("--module-deps", ctx.actions.write("module_deps_for_{}".format(module), apk_module_graph_info.module_to_module_deps_function(module)))

            multi_dex_cmd.add("--module", module)
            multi_dex_cmd.add("--canary-class-name", apk_module_graph_info.module_to_canary_class_name_function(module))

            jar_to_dex_file = ctx.actions.write("jars_to_dex_file_for_module_{}.txt".format(module), jars)
            multi_dex_cmd.add("--files-to-dex-list", jar_to_dex_file)
            multi_dex_cmd.hidden(jars)

            multi_dex_cmd.add("--android-jar", android_toolchain.android_jar)
            if not is_optimized:
                multi_dex_cmd.add("--no-optimize")

            if proguard_configuration_output_file:
                multi_dex_cmd.add("--proguard-configuration-file", proguard_configuration_output_file)
                multi_dex_cmd.add("--proguard-mapping-file", proguard_mapping_output_file)

            multi_dex_cmd.add("--compression", _get_dex_compression(ctx))
            multi_dex_cmd.add("--xz-compression-level", str(ctx.attrs.xz_compression_level))
            if ctx.attrs.minimize_primary_dex_size:
                multi_dex_cmd.add("--minimize-primary-dex")

            ctx.actions.run(multi_dex_cmd, category = "multi_dex", identifier = "{}:{}_module_{}".format(ctx.label.package, ctx.label.name, module))

        ctx.actions.symlinked_dir(ctx.outputs[secondary_dex_dir], secondary_dex_dir_srcs)

    ctx.actions.dynamic_output(inputs, [], outputs, do_multi_dex)

    return DexFilesInfo(
        primary_dex = primary_dex_file,
        secondary_dex_dirs = [root_module_secondary_dex_output_dir, secondary_dex_dir],
        proguard_text_files_path = None,
    )

def merge_to_single_dex(
        ctx: "context",
        android_toolchain: "AndroidToolchainInfo",
        pre_dexed_libs: ["DexLibraryInfo"]) -> "DexFilesInfo":
    output_dex_file = ctx.actions.declare_output("classes.dex")
    pre_dexed_artifacts_to_dex_file = ctx.actions.declare_output("pre_dexed_artifacts_to_dex_file.txt")
    pre_dexed_artifacts = [pre_dexed_lib.dex for pre_dexed_lib in pre_dexed_libs if pre_dexed_lib.dex != None]
    _merge_dexes(ctx, android_toolchain, output_dex_file, pre_dexed_artifacts, pre_dexed_artifacts_to_dex_file)

    return DexFilesInfo(
        primary_dex = output_dex_file,
        secondary_dex_dirs = [],
        proguard_text_files_path = None,
    )

DexInputWithSpecifiedClasses = record(
    lib = "DexLibraryInfo",
    dex_class_names = [str.type],
)

DexInputWithClassNamesFile = record(
    lib = "DexLibraryInfo",
    filtered_class_names_file = "artifact",
)

# When using jar compression, the secondary dex directory consists of N secondary dex jars, each
# of which has a corresponding .meta file (the secondary_dex_metadata_file) containing a single
# line of the form:
# jar:<size of secondary dex jar (in bytes)> dex:<size of uncompressed dex file (in bytes)>
#
# It also contains a metadata.txt file, which consists on N lines, one for each secondary dex
# jar. Those lines consist of:
# <secondary dex file name> <sha1 hash of secondary dex> <canary class>
#
# We write the line that needs to be added to metadata.txt for this secondary dex jar to
# secondary_dex_metadata_line, and we use the secondary_dex_canary_class_name for the
# <canary class>.
#
# When we have finished building all of the secondary dexes, we read each of the
# secondary_dex_metadata_line artifacts and write them to a single metadata.txt file.
# We do that for raw compression too, since it also has a metadata.txt file.
SecondaryDexMetadataConfig = record(
    secondary_dex_compression = str.type,
    secondary_dex_metadata_path = [str.type, None],
    secondary_dex_metadata_file = ["artifact", None],
    secondary_dex_metadata_line = "artifact",
    secondary_dex_canary_class_name = str.type,
)

def _get_secondary_dex_jar_metadata_config(
        actions: "actions",
        secondary_dex_path: str.type,
        module: str.type,
        module_to_canary_class_name_function: "function",
        index: int.type) -> SecondaryDexMetadataConfig.type:
    secondary_dex_metadata_path = secondary_dex_path + ".meta"
    return SecondaryDexMetadataConfig(
        secondary_dex_compression = "jar",
        secondary_dex_metadata_path = secondary_dex_metadata_path,
        secondary_dex_metadata_file = actions.declare_output(secondary_dex_metadata_path),
        secondary_dex_metadata_line = actions.declare_output("metadata_line_artifacts/{}/{}".format(module, index + 1)),
        secondary_dex_canary_class_name = _get_fully_qualified_canary_class_name(module, module_to_canary_class_name_function, index + 1),
    )

def _get_secondary_dex_raw_metadata_config(
        actions: "actions",
        module: str.type,
        module_to_canary_class_name_function: "function",
        index: int.type) -> SecondaryDexMetadataConfig.type:
    return SecondaryDexMetadataConfig(
        secondary_dex_compression = "raw",
        secondary_dex_metadata_path = None,
        secondary_dex_metadata_file = None,
        secondary_dex_metadata_line = actions.declare_output("metadata_line_artifacts/{}/{}".format(module, index + 1)),
        secondary_dex_canary_class_name = _get_fully_qualified_canary_class_name(module, module_to_canary_class_name_function, index + 1),
    )

def _get_filter_dex_batch_size() -> int.type:
    return 100

def _filter_pre_dexed_libs(
        actions: "actions",
        android_toolchain: "AndroidToolchainInfo",
        primary_dex_patterns_file: "artifact",
        pre_dexed_libs: ["DexLibraryInfo"],
        batch_number: int.type) -> [DexInputWithClassNamesFile.type]:
    pre_dexed_lib_with_class_names_files = []
    for pre_dexed_lib in pre_dexed_libs:
        class_names = pre_dexed_lib.class_names
        id = "{}_{}_{}".format(class_names.owner.package, class_names.owner.name, class_names.short_path)
        filtered_class_names_file = actions.declare_output("primary_dex_class_names_for_{}".format(id))
        pre_dexed_lib_with_class_names_files.append(
            DexInputWithClassNamesFile(lib = pre_dexed_lib, filtered_class_names_file = filtered_class_names_file),
        )

    filter_dex_cmd = cmd_args([
        android_toolchain.filter_dex_class_names[RunInfo],
        "--primary-dex-patterns",
        primary_dex_patterns_file,
        "--class-names",
        [x.lib.class_names for x in pre_dexed_lib_with_class_names_files],
        "--output",
        [x.filtered_class_names_file.as_output() for x in pre_dexed_lib_with_class_names_files],
    ])
    actions.run(filter_dex_cmd, category = "filter_dex", identifier = "batch_{}".format(batch_number))

    return pre_dexed_lib_with_class_names_files

_SortedPreDexedInputs = record(
    module = str.type,
    primary_dex_inputs = [DexInputWithSpecifiedClasses.type],
    secondary_dex_inputs = [[DexInputWithSpecifiedClasses.type]],
)

def merge_to_split_dex(
        ctx: "context",
        android_toolchain: "AndroidToolchainInfo",
        pre_dexed_libs: ["DexLibraryInfo"],
        split_dex_merge_config: "SplitDexMergeConfig",
        apk_module_graph_file: ["artifact", None] = None) -> "DexFilesInfo":
    primary_dex_patterns_file = ctx.actions.write("primary_dex_patterns_file", split_dex_merge_config.primary_dex_patterns)

    pre_dexed_lib_with_class_names_files = []

    batch_size = _get_filter_dex_batch_size()
    for (batch_number, start_index) in enumerate(range(0, len(pre_dexed_libs), batch_size)):
        end_index = min(start_index + batch_size, len(pre_dexed_libs))
        pre_dexed_lib_with_class_names_files.extend(
            _filter_pre_dexed_libs(
                ctx.actions,
                android_toolchain,
                primary_dex_patterns_file,
                pre_dexed_libs[start_index:end_index],
                batch_number,
            ),
        )

    input_artifacts = flatten([[
        input.lib.dex,
        input.lib.weight_estimate,
        input.filtered_class_names_file,
    ] for input in pre_dexed_lib_with_class_names_files]) + ([apk_module_graph_file] if apk_module_graph_file else [])
    primary_dex_artifact_list = ctx.actions.declare_output("pre_dexed_artifacts_for_primary_dex.txt")
    primary_dex_output = ctx.actions.declare_output("classes.dex")
    secondary_dexes_dir = ctx.actions.declare_output("secondary_dexes_dir")

    outputs = [primary_dex_output, primary_dex_artifact_list, secondary_dexes_dir]

    def merge_pre_dexed_libs(ctx: "context"):
        apk_module_graph_info = get_apk_module_graph_info(ctx, apk_module_graph_file) if apk_module_graph_file else get_root_module_only_apk_module_graph_info()
        module_to_canary_class_name_function = apk_module_graph_info.module_to_canary_class_name_function
        sorted_pre_dexed_inputs = _sort_pre_dexed_files(
            ctx,
            pre_dexed_lib_with_class_names_files,
            split_dex_merge_config,
            get_module_from_target = apk_module_graph_info.target_to_module_mapping_function,
            module_to_canary_class_name_function = module_to_canary_class_name_function,
        )
        secondary_dexes_for_symlinking = {}
        metadata_line_artifacts_by_module = {}
        metadata_dot_txt_files_by_module = {}

        for sorted_pre_dexed_input in sorted_pre_dexed_inputs:
            module = sorted_pre_dexed_input.module
            primary_dex_inputs = sorted_pre_dexed_input.primary_dex_inputs
            pre_dexed_artifacts = [primary_dex_input.lib.dex for primary_dex_input in primary_dex_inputs if primary_dex_input.lib.dex]
            if pre_dexed_artifacts:
                expect(is_root_module(module), "module {} should not have a primary dex!".format(module))
                primary_dex_class_list = ctx.actions.write(
                    "class_list_for_primary_dex.txt",
                    flatten([primary_dex_input.dex_class_names for primary_dex_input in primary_dex_inputs]),
                )

                _merge_dexes(
                    ctx,
                    android_toolchain,
                    ctx.outputs[primary_dex_output],
                    pre_dexed_artifacts,
                    ctx.outputs[primary_dex_artifact_list],
                    class_names_to_include = primary_dex_class_list,
                )

            secondary_dex_inputs = sorted_pre_dexed_input.secondary_dex_inputs
            raw_secondary_dexes_for_compressing = {}
            for i in range(len(secondary_dex_inputs)):
                if split_dex_merge_config.dex_compression == "jar" or split_dex_merge_config.dex_compression == "raw":
                    if split_dex_merge_config.dex_compression == "jar":
                        secondary_dex_path = _get_jar_secondary_dex_path(i, module)
                        secondary_dex_metadata_config = _get_secondary_dex_jar_metadata_config(ctx.actions, secondary_dex_path, module, module_to_canary_class_name_function, i)
                        secondary_dexes_for_symlinking[secondary_dex_metadata_config.secondary_dex_metadata_path] = secondary_dex_metadata_config.secondary_dex_metadata_file
                    else:
                        secondary_dex_path = _get_raw_secondary_dex_path(i, module)
                        secondary_dex_metadata_config = _get_secondary_dex_raw_metadata_config(ctx.actions, module, module_to_canary_class_name_function, i)

                    secondary_dex_output = ctx.actions.declare_output(secondary_dex_path)
                    secondary_dexes_for_symlinking[secondary_dex_path] = secondary_dex_output
                    metadata_line_artifacts_by_module.setdefault(module, []).append(secondary_dex_metadata_config.secondary_dex_metadata_line)
                else:
                    secondary_dex_name = _get_raw_secondary_dex_name(i, module)
                    secondary_dex_output = ctx.actions.declare_output("{}/{}".format(module, secondary_dex_name))
                    raw_secondary_dexes_for_compressing[secondary_dex_name] = secondary_dex_output
                    secondary_dex_metadata_config = None

                secondary_dex_artifact_list = ctx.actions.declare_output("pre_dexed_artifacts_for_secondary_dex_{}_for_module_{}.txt".format(i + 2, module))
                secondary_dex_class_list = ctx.actions.write(
                    "class_list_for_secondary_dex_{}_for_module_{}.txt".format(i + 2, module),
                    flatten([secondary_dex_input.dex_class_names for secondary_dex_input in secondary_dex_inputs[i]]),
                )
                pre_dexed_artifacts = [secondary_dex_input.lib.dex for secondary_dex_input in secondary_dex_inputs[i] if secondary_dex_input.lib.dex]
                _merge_dexes(
                    ctx,
                    android_toolchain,
                    secondary_dex_output,
                    pre_dexed_artifacts,
                    secondary_dex_artifact_list,
                    class_names_to_include = secondary_dex_class_list,
                    secondary_dex_metadata_config = secondary_dex_metadata_config,
                )

            if split_dex_merge_config.dex_compression == "jar" or split_dex_merge_config.dex_compression == "raw":
                metadata_dot_txt_path = "{}/metadata.txt".format(_get_secondary_dex_subdir(module))
                metadata_dot_txt_file = ctx.actions.declare_output(metadata_dot_txt_path)
                secondary_dexes_for_symlinking[metadata_dot_txt_path] = metadata_dot_txt_file
                metadata_dot_txt_files_by_module[module] = metadata_dot_txt_file
            else:
                raw_secondary_dexes_dir = ctx.actions.symlinked_dir("raw_secondary_dexes_dir_for_module_{}".format(module), raw_secondary_dexes_for_compressing)
                secondary_dex_dir_for_module = ctx.actions.declare_output("secondary_dexes_dir_for_{}".format(module))
                secondary_dex_subdir = secondary_dex_dir_for_module.project(_get_secondary_dex_subdir(module))

                multi_dex_cmd = cmd_args(android_toolchain.multi_dex_command[RunInfo])
                multi_dex_cmd.add("--secondary-dex-output-dir", secondary_dex_dir_for_module.as_output())
                multi_dex_cmd.add("--raw-secondary-dexes-dir", raw_secondary_dexes_dir)
                multi_dex_cmd.add("--compression", _get_dex_compression(ctx))
                multi_dex_cmd.add("--xz-compression-level", str(ctx.attrs.xz_compression_level))
                multi_dex_cmd.add("--module", module)
                multi_dex_cmd.add("--canary-class-name", module_to_canary_class_name_function(module))
                if not is_root_module(module):
                    multi_dex_cmd.add("--module-deps", ctx.actions.write("module_deps_for_{}".format(module), apk_module_graph_info.module_to_module_deps_function(module)))

                ctx.actions.run(multi_dex_cmd, category = "multi_dex_from_raw_dexes", identifier = "{}:{}_module_{}".format(ctx.label.package, ctx.label.name, module))

                secondary_dexes_for_symlinking[_get_secondary_dex_subdir(module)] = secondary_dex_subdir

        if metadata_dot_txt_files_by_module:
            def write_metadata_dot_txts(ctx: "context"):
                for voltron_module, metadata_dot_txt in metadata_dot_txt_files_by_module.items():
                    metadata_line_artifacts = metadata_line_artifacts_by_module[voltron_module]
                    expect(metadata_line_artifacts != None, "Should have metadata lines!")

                    metadata_lines = [".id {}".format(voltron_module)]
                    metadata_lines.extend([".requires {}".format(module_dep) for module_dep in apk_module_graph_info.module_to_module_deps_function(voltron_module)])
                    if split_dex_merge_config.dex_compression == "raw" and is_root_module(voltron_module):
                        metadata_lines.append(".root_relative")
                    for metadata_line_artifact in metadata_line_artifacts:
                        metadata_lines.append(ctx.artifacts[metadata_line_artifact].read_string().strip())
                    ctx.actions.write(ctx.outputs[metadata_dot_txt], metadata_lines)

            ctx.actions.dynamic_output(flatten(metadata_line_artifacts_by_module.values()), [], metadata_dot_txt_files_by_module.values(), write_metadata_dot_txts)

        ctx.actions.symlinked_dir(
            ctx.outputs[secondary_dexes_dir],
            secondary_dexes_for_symlinking,
        )

    ctx.actions.dynamic_output(input_artifacts, [], outputs, merge_pre_dexed_libs)

    return DexFilesInfo(
        primary_dex = primary_dex_output,
        secondary_dex_dirs = [secondary_dexes_dir],
        proguard_text_files_path = None,
    )

def _merge_dexes(
        ctx: "context",
        android_toolchain: "AndroidToolchainInfo",
        output_dex_file: "artifact",
        pre_dexed_artifacts: ["artifact"],
        pre_dexed_artifacts_file: "artifact",
        class_names_to_include: ["artifact", None] = None,
        secondary_output_dex_file: ["artifact", None] = None,
        secondary_dex_metadata_config: [SecondaryDexMetadataConfig.type, None] = None):
    d8_cmd = cmd_args(android_toolchain.d8_command[RunInfo])
    d8_cmd.add(["--output-dex-file", output_dex_file.as_output()])

    pre_dexed_artifacts_to_dex_file = ctx.actions.write(pre_dexed_artifacts_file.as_output(), pre_dexed_artifacts)
    d8_cmd.add(["--files-to-dex-list", pre_dexed_artifacts_to_dex_file])
    d8_cmd.hidden(pre_dexed_artifacts)

    d8_cmd.add(["--android-jar", android_toolchain.android_jar])
    d8_cmd.add(_DEX_MERGE_OPTIONS)

    if class_names_to_include:
        d8_cmd.add(["--primary-dex-class-names-path", class_names_to_include])

    if secondary_output_dex_file:
        d8_cmd.add(["--secondary-output-dex-file", secondary_output_dex_file.as_output()])

    if secondary_dex_metadata_config:
        d8_cmd.add(["--secondary-dex-compression", secondary_dex_metadata_config.secondary_dex_compression])
        if secondary_dex_metadata_config.secondary_dex_metadata_file:
            d8_cmd.add(["--secondary-dex-metadata-file", secondary_dex_metadata_config.secondary_dex_metadata_file.as_output()])
        d8_cmd.add(["--secondary-dex-metadata-line", secondary_dex_metadata_config.secondary_dex_metadata_line.as_output()])
        d8_cmd.add(["--secondary-dex-canary-class-name", secondary_dex_metadata_config.secondary_dex_canary_class_name])

    ctx.actions.run(
        d8_cmd,
        category = "d8",
        identifier = "{}:{} {}".format(ctx.label.package, ctx.label.name, output_dex_file.short_path),
    )

def _sort_pre_dexed_files(
        ctx: "context",
        pre_dexed_lib_with_class_names_files: ["DexInputWithClassNamesFile"],
        split_dex_merge_config: "SplitDexMergeConfig",
        get_module_from_target: "function",
        module_to_canary_class_name_function: "function") -> [_SortedPreDexedInputs.type]:
    sorted_pre_dexed_inputs_map = {}
    current_secondary_dex_size_map = {}
    current_secondary_dex_inputs_map = {}
    for pre_dexed_lib_with_class_names_file in pre_dexed_lib_with_class_names_files:
        pre_dexed_lib = pre_dexed_lib_with_class_names_file.lib
        module = get_module_from_target(str(pre_dexed_lib.dex.owner.raw_target()))
        primary_dex_data, secondary_dex_data = ctx.artifacts[pre_dexed_lib_with_class_names_file.filtered_class_names_file].read_string().split(";")
        primary_dex_class_names = primary_dex_data.split(",") if primary_dex_data else []
        secondary_dex_class_names = secondary_dex_data.split(",") if secondary_dex_data else []

        module_pre_dexed_inputs = sorted_pre_dexed_inputs_map.setdefault(module, _SortedPreDexedInputs(
            module = module,
            primary_dex_inputs = [],
            secondary_dex_inputs = [],
        ))

        primary_dex_inputs = module_pre_dexed_inputs.primary_dex_inputs
        secondary_dex_inputs = module_pre_dexed_inputs.secondary_dex_inputs

        if len(primary_dex_class_names) > 0:
            expect(
                is_root_module(module),
                "Non-root modules should not have anything that belongs in the primary dex, " +
                "but {} is assigned to module {} and has the following class names in the primary dex: {}\n".format(
                    pre_dexed_lib.dex.owner,
                    module,
                    "\n".join(primary_dex_class_names),
                ),
            )
            primary_dex_inputs.append(
                DexInputWithSpecifiedClasses(lib = pre_dexed_lib, dex_class_names = primary_dex_class_names),
            )

        if len(secondary_dex_class_names) > 0:
            weight_estimate = int(ctx.artifacts[pre_dexed_lib.weight_estimate].read_string().strip())
            current_secondary_dex_size = current_secondary_dex_size_map.get(module, 0)
            if current_secondary_dex_size + weight_estimate > split_dex_merge_config.secondary_dex_weight_limit_bytes:
                current_secondary_dex_size = 0
                current_secondary_dex_inputs_map[module] = []

            current_secondary_dex_inputs = current_secondary_dex_inputs_map.setdefault(module, [])
            if len(current_secondary_dex_inputs) == 0:
                canary_class_dex_input = _create_canary_class(
                    ctx,
                    len(secondary_dex_inputs) + 1,
                    module,
                    module_to_canary_class_name_function,
                    ctx.attrs._dex_toolchain[DexToolchainInfo],
                )
                current_secondary_dex_inputs.append(canary_class_dex_input)
                secondary_dex_inputs.append(current_secondary_dex_inputs)

            current_secondary_dex_size_map[module] = current_secondary_dex_size + weight_estimate
            current_secondary_dex_inputs.append(
                DexInputWithSpecifiedClasses(lib = pre_dexed_lib, dex_class_names = secondary_dex_class_names),
            )

    return sorted_pre_dexed_inputs_map.values()

def _get_raw_secondary_dex_name(index: int.type, module: str.type) -> str.type:
    # Root module begins at 2 (primary classes.dex is 1)
    # Non-root module begins at 1 (classes.dex)
    if is_root_module(module):
        return "classes{}.dex".format(index + 2)
    elif index == 0:
        return "classes.dex".format(module)
    else:
        return "classes{}.dex".format(module, index + 1)

def _get_raw_secondary_dex_path(index: int.type, module: str.type):
    if is_root_module(module):
        return _get_raw_secondary_dex_name(index, module)
    else:
        return "assets/{}/{}".format(module, _get_raw_secondary_dex_name(index, module))

def _get_jar_secondary_dex_path(index: int.type, module: str.type):
    return "{}/{}-{}.dex.jar".format(
        _get_secondary_dex_subdir(module),
        "secondary" if is_root_module(module) else module,
        index + 1,
    )

def _get_secondary_dex_subdir(module: str.type):
    return "assets/{}".format("secondary-program-dex-jars" if is_root_module(module) else module)

# We create "canary" classes and add them to each secondary dex jar to ensure each jar has a class
# that can be safely loaded on any system. This class is used during secondary dex verification.
_CANARY_FULLY_QUALIFIED_CLASS_NAME_TEMPLATE = "{}.dex{}.Canary"
_CANARY_FILE_NAME_TEMPLATE = "canary_classes/{}/dex{}/Canary.java"
_CANARY_CLASS_PACKAGE_TEMPLATE = "package {}.dex{};\n"
_CANARY_CLASS_INTERFACE_DEFINITION = "public interface Canary {}"

def _create_canary_class(
        ctx: "context",
        index: int.type,
        module: str.type,
        module_to_canary_class_name_function: "function",
        dex_toolchain: DexToolchainInfo.type) -> DexInputWithSpecifiedClasses.type:
    prefix = module_to_canary_class_name_function(module)
    canary_class_java_file = ctx.actions.write(_CANARY_FILE_NAME_TEMPLATE.format(prefix, index), [_CANARY_CLASS_PACKAGE_TEMPLATE.format(prefix, index), _CANARY_CLASS_INTERFACE_DEFINITION])
    canary_class_jar = ctx.actions.declare_output("canary_classes/{}/canary_jar_{}.jar".format(prefix, index))
    compile_to_jar(ctx, [canary_class_java_file], output = canary_class_jar, actions_prefix = "{}_canary_class{}".format(prefix, index))

    dex_library_info = get_dex_produced_from_java_library(ctx, dex_toolchain = dex_toolchain, jar_to_dex = canary_class_jar)

    return DexInputWithSpecifiedClasses(
        lib = dex_library_info,
        dex_class_names = [_get_fully_qualified_canary_class_name(module, module_to_canary_class_name_function, index).replace(".", "/") + ".class"],
    )

def _get_fully_qualified_canary_class_name(module: str.type, module_to_canary_class_name_function: "function", index: int.type) -> str.type:
    prefix = module_to_canary_class_name_function(module)
    return _CANARY_FULLY_QUALIFIED_CLASS_NAME_TEMPLATE.format(prefix, index)
