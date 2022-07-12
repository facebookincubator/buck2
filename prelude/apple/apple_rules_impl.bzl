load("@fbcode//buck2/prelude:attributes.bzl", "LinkableDepType", "Linkage")
load("@fbcode//buck2/prelude/cxx:headers.bzl", "CPrecompiledHeaderInfo")
load(":apple_asset_catalog.bzl", "apple_asset_catalog_impl")
load(":apple_binary.bzl", "apple_binary_impl")
load(":apple_bundle.bzl", "apple_bundle_impl")
load(":apple_bundle_types.bzl", "AppleBundleResourceInfo")
load(":apple_code_signing_types.bzl", "CodeSignType")
load(":apple_core_data.bzl", "apple_core_data_impl")
load(":apple_library.bzl", "apple_library_impl")
load(":apple_package.bzl", "apple_package_impl")
load(":apple_resource.bzl", "apple_resource_impl")
load(":apple_test.bzl", "apple_test_impl")
load(":apple_toolchain.bzl", "apple_toolchain_impl")
load(":apple_toolchain_types.bzl", "AppleToolchainInfo", "AppleToolsInfo")
load(":prebuilt_apple_framework.bzl", "prebuilt_apple_framework_impl")
load(":swift_toolchain.bzl", "swift_toolchain_impl")
load(":xcode_prebuild_script.bzl", "xcode_prebuild_script_impl")

def _get_apple_tolchain_attr():
    return attr.toolchain_dep(default = "fbcode//buck2/platform/toolchain:apple-default", providers = [AppleToolchainInfo])

implemented_rules = {
    "apple_asset_catalog": apple_asset_catalog_impl,
    "apple_binary": apple_binary_impl,
    "apple_bundle": apple_bundle_impl,
    "apple_library": apple_library_impl,
    "apple_package": apple_package_impl,
    "apple_resource": apple_resource_impl,
    "apple_test": apple_test_impl,
    "apple_toolchain": apple_toolchain_impl,
    "core_data_model": apple_core_data_impl,
    "prebuilt_apple_framework": prebuilt_apple_framework_impl,
    "swift_toolchain": swift_toolchain_impl,
    "xcode_prebuild_script": xcode_prebuild_script_impl,
}

extra_attributes = {
    "apple_asset_catalog": {
        "dirs": attr.list(attr.source(allow_directory = True), default = []),
    },
    "apple_binary": {
        "enable_distributed_thinlto": attr.bool(default = False),
        "extra_xcode_sources": attr.list(attr.source(allow_directory = True), default = []),
        "precompiled_header": attr.option(attr.dep(providers = [CPrecompiledHeaderInfo]), default = None),
        "prefer_stripped_objects": attr.bool(default = False),
        "preferred_linkage": attr.enum(Linkage, default = "any"),
        "stripped": attr.bool(default = False),
        "_apple_toolchain": _get_apple_tolchain_attr(),
    },
    "apple_bundle": {
        "_apple_installer": attr.exec_dep(default = "buck//src/com/facebook/buck/installer:apple_installer", providers = [RunInfo]),
        "_apple_toolchain": _get_apple_tolchain_attr(),
        "_apple_tools": attr.exec_dep(default = "fbsource//xplat/buck2/platform/apple:apple-tools", providers = [AppleToolsInfo]),
        "_codesign_entitlements": attr.option(attr.source(), default = None),
        "_codesign_type": attr.option(attr.enum(CodeSignType.values()), default = None),
        "_incremental_bundling_enabled": attr.bool(),
        "_provisioning_profiles": attr.dep(default = "fbsource//xplat/buck2/provisioning_profiles:all"),
        "_resource_bundle": attr.option(attr.dep(providers = [AppleBundleResourceInfo]), default = None),
    },
    "apple_library": {
        "extra_xcode_sources": attr.list(attr.source(allow_directory = True), default = []),
        "precompiled_header": attr.option(attr.dep(providers = [CPrecompiledHeaderInfo]), default = None),
        "preferred_linkage": attr.enum(Linkage, default = "any"),
        "stripped": attr.bool(default = False),
        "use_archive": attr.option(attr.bool(), default = None),
        "_apple_toolchain": _get_apple_tolchain_attr(),
        "_apple_tools": attr.exec_dep(default = "fbsource//xplat/buck2/platform/apple:apple-tools", providers = [AppleToolsInfo]),
    },
    "apple_resource": {
        "codesign_on_copy": attr.bool(default = False),
        "content_dirs": attr.list(attr.source(allow_directory = True), default = []),
        "dirs": attr.list(attr.source(allow_directory = True), default = []),
    },
    # To build an `apple_test`, one needs to first build a shared `apple_library` then
    # wrap this test library into an `apple_bundle`. Because of this, `apple_test` has attributes
    # from both `apple_library` and `apple_bundle`.
    "apple_test": {
        # Expected by `apple_bundle`, for `apple_test` this field is always None.
        "binary": attr.option(attr.dep(), default = None),
        # The resulting test bundle should have .xctest extension.
        "extension": attr.string(default = "xctest"),
        "extra_xcode_sources": attr.list(attr.source(allow_directory = True), default = []),
        # Used to create the shared test library. Any library deps whose `preferred_linkage` isn't "shared" will
        # be treated as "static" deps and linked into the shared test library.
        "link_style": attr.enum(LinkableDepType, default = "static"),
        # The test source code and lib dependencies should be built into a shared library.
        "preferred_linkage": attr.enum(Linkage, default = "shared"),
        # Expected by `apple_bundle`, for `apple_test` this field is always None.
        "resource_group": attr.option(attr.string(), default = None),
        # Expected by `apple_bundle`, for `apple_test` this field is always None.
        "resource_group_map": attr.option(attr.string(), default = None),
        "stripped": attr.bool(default = False),
        "_apple_toolchain": _get_apple_tolchain_attr(),
        "_apple_tools": attr.exec_dep(default = "fbsource//xplat/buck2/platform/apple:apple-tools", providers = [AppleToolsInfo]),
        "_codesign_type": attr.option(attr.enum(CodeSignType.values()), default = None),
        "_incremental_bundling_enabled": attr.bool(),
    },
    "apple_toolchain": {
        # The Buck v1 attribute specs defines those as `attr.source()` but
        # we want to properly handle any runnable tools that might have
        # addition runtime requirements.
        "actool": attr.dep(providers = [RunInfo]),
        "codesign": attr.dep(providers = [RunInfo]),
        "codesign_allocate": attr.dep(providers = [RunInfo]),
        # Controls invocations of `ibtool`, `actool` and `momc`
        "compile_resources_locally": attr.bool(default = False),
        "dsymutil": attr.dep(providers = [RunInfo]),
        "dwarfdump": attr.option(attr.dep(providers = [RunInfo]), default = None),
        "ibtool": attr.dep(providers = [RunInfo]),
        "libtool": attr.dep(providers = [RunInfo]),
        "lipo": attr.dep(providers = [RunInfo]),
        "min_version": attr.option(attr.string(), default = None),
        "momc": attr.dep(providers = [RunInfo]),
        "platform_path": attr.option(attr.source()),  # Mark as optional until we remove `_internal_platform_path`
        "sdk_path": attr.option(attr.source()),  # Mark as optional until we remove `_internal_sdk_path`
        "version": attr.option(attr.string(), default = None),
        "xcode_build_version": attr.option(attr.string(), default = None),
        "xcode_version": attr.option(attr.string(), default = None),
        "xctest": attr.dep(providers = [RunInfo]),
        # TODO(T111858757): Mirror of `platform_path` but treated as a string. It allows us to
        #                   pass abs paths during development and using the currently selected Xcode.
        "_internal_platform_path": attr.option(attr.string()),
        # TODO(T111858757): Mirror of `sdk_path` but treated as a string. It allows us to
        #                   pass abs paths during development and using the currently selected Xcode.
        "_internal_sdk_path": attr.option(attr.string()),
    },
    "core_data_model": {
        "path": attr.source(allow_directory = True),
    },
    "prebuilt_apple_framework": {
        "framework": attr.option(attr.source(allow_directory = True), default = None),
        "preferred_linkage": attr.enum(Linkage, default = "any"),
        "_apple_toolchain": _get_apple_tolchain_attr(),
    },
    "scene_kit_assets": {
        "path": attr.source(allow_directory = True),
    },
    "swift_library": {
        "preferred_linkage": attr.enum(Linkage, default = "any"),
    },
    "swift_toolchain": {
        "architecture": attr.option(attr.string(), default = None),  # TODO(T115173356): Make field non-optional
        "platform_path": attr.option(attr.source()),  # Mark as optional until we remove `_internal_platform_path`
        "sdk_modules": attr.list(attr.dep(), default = []),  # A list or a root target that represent a graph of sdk modules (e.g Frameworks)
        "sdk_path": attr.option(attr.source()),  # Mark as optional until we remove `_internal_sdk_path`
        "swift_stdlib_tool": attr.exec_dep(providers = [RunInfo]),
        "swiftc": attr.exec_dep(providers = [RunInfo]),
        # TODO(T111858757): Mirror of `platform_path` but treated as a string. It allows us to
        #                   pass abs paths during development and using the currently selected Xcode.
        "_internal_platform_path": attr.option(attr.string(), default = None),
        # TODO(T111858757): Mirror of `sdk_path` but treated as a string. It allows us to
        #                   pass abs paths during development and using the currently selected Xcode.
        "_internal_sdk_path": attr.option(attr.string(), default = None),
        "_swiftc_wrapper": attr.dep(providers = [RunInfo], default = "@fbcode//buck2/prelude/apple/tools:swift_exec"),
    },
}
