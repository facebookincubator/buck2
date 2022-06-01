AndroidPlatformInfo = provider(fields = [
    "name",
])

AndroidToolchainInfo = provider(fields = [
    "aapt2",
    "aidl",
    "android_jar",
    "android_bootclasspath",
    "apk_builder",
    "d8_command",
    "multi_dex_command",
    "framework_aidl_file",
    "generate_build_config",
    "generate_manifest",
    "instrumentation_test_runner_classpath",
    "instrumentation_test_runner_main_class",
    "manifest_utils",
    "merge_android_resources",
    "merge_assets",
    "mini_aapt",
    "optimized_proguard_config",
    "proguard_config",
    "proguard_jar",
    "proguard_max_heap_size",
    "secondary_dex_weight_limit",
    "unpack_aar",
    "zipalign",
])
