load("@fbcode//buck2/platform:utils.bzl", "string_attr", "string_list_attr")
load("@fbcode//buck2/prelude/ocaml:providers.bzl", "OCamlPlatformInfo", "OCamlToolchainInfo")

# Got these attributes from
# 'fbcode/tools/buckconfigs/fbcode/modes/dev-sand.bcfg'
_buckconfig_ocaml_toolchain_attrs = {
    "debug": (string_attr, ""),
    "dep.tool": (string_attr, ""),
    "interop.includes": (string_attr, ""),
    "lex.compiler": (string_attr, ""),
    "ocaml.bytecode.compiler": (string_attr, ""),
    "ocaml.compiler": (string_attr, ""),
    "ocaml_compiler_flags": (string_list_attr, []),
    "warnings_flags": (string_attr, ""),
    "yacc.compiler": (string_attr, ""),
}

# The values we read from the buck config (e.g.
# 'fbcode/tools/buckconfigs/fbcode/modes/dev-sand.bcfg') are file
# paths but what we actually need are the buck targets that correspond
# to those paths (i.e. the `buck_sh_binary` targets as defined in
# 'fbcode/third-party-buck/platform009/build/supercaml/TARGETS').
# Thus, we define a dictionary and use it map from one to the other.
def pathfix(platform, val):
    # TODO(2021-08-26; ndmitchell, shaynefletcher): This whole mapping
    # table business we don't want in the long-term. It will do for
    # the next few months but let's fix it when it breaks (see
    # T99095335).

    # Don't attempt lookups on non-string values.
    if type(val) != type(""):
        return val

    paths = {
        # ocaml_bytecode_compiler
        "./third-party-buck/" + platform + "/build/supercaml/share/dotopam/default/bin/ocamlc.opt": "fbcode//third-party-buck/" + platform + "/build/supercaml:ocamlc-exe",
        # debug (there is no 'ocamldebug' in bin; sub with 'ocamlc')
        "./third-party-buck/" + platform + "/build/supercaml/share/dotopam/default/bin/ocamldebug": "fbcode//third-party-buck/" + platform + "/build/supercaml:ocamlc-exe",
        # dep_tool
        "./third-party-buck/" + platform + "/build/supercaml/share/dotopam/default/bin/ocamldep.opt": "fbcode//third-party-buck/" + platform + "/build/supercaml:ocamldep-exe",
        # lex_compiler
        "./third-party-buck/" + platform + "/build/supercaml/share/dotopam/default/bin/ocamllex.opt": "fbcode//third-party-buck/" + platform + "/build/supercaml:ocamllex-exe",
        # ocaml_compiler
        "./third-party-buck/" + platform + "/build/supercaml/share/dotopam/default/bin/ocamlopt.opt": "fbcode//third-party-buck/" + platform + "/build/supercaml:ocamlopt-exe",
        # yacc_compiler
        "./third-party-buck/" + platform + "/build/supercaml/share/dotopam/default/bin/ocamlyacc": "fbcode//third-party-buck/" + platform + "/build/supercaml:ocamlyacc-exe",
        # interop_includes (this is a location not a binary!; fix this (somehow) later)
        "./third-party-buck/" + platform + "/build/supercaml/share/dotopam/default/lib/ocaml": "fbcode//third-party-buck/" + platform + "/build/supercaml:interop_includes",
    }

    return paths.get(val, val)

def config_backed_ocaml_toolchain(flavor, **kwargs):
    flavor_bits = flavor.split("-")
    platform = flavor_bits[0]

    # Special case: "platform010-compat"
    if flavor_bits[1] == "compat":
        platform += "-" + flavor_bits[1]

    # Special case: "platform010-aarch64"
    if flavor_bits[1] == "aarch64":
        platform += "-" + flavor_bits[1]

    sections = ["ocaml#" + flavor, "ocaml"]
    for (key, (info, default)) in _buckconfig_ocaml_toolchain_attrs.items():
        if key in kwargs:
            continue
        val = None
        for section in sections:
            val = info.reader(section, key)
            if val != None:
                break
        if val == None:
            val = default
        kwargs[key.replace(".", "_")] = pathfix(platform, val)

    _config_backed_ocaml_toolchain_rule(
        name = "ocaml-" + flavor,
        **kwargs
    )

def _config_backed_ocaml_toolchain_rule_impl(ctx):
    return [
        DefaultInfo(),
        OCamlToolchainInfo(
            debug = ctx.attrs.debug[RunInfo],
            binutils_ld = ctx.attrs.binutils_ld,
            dep_tool = ctx.attrs.dep_tool[RunInfo],
            interop_includes = ctx.attrs.interop_includes,
            lex_compiler = ctx.attrs.lex_compiler[RunInfo],
            libasmrun = ctx.attrs.libasmrun,
            ocaml_bytecode_compiler = ctx.attrs.ocaml_bytecode_compiler[RunInfo],
            ocaml_compiler = ctx.attrs.ocaml_compiler[RunInfo],
            warnings_flags = ctx.attrs.warnings_flags,
            ocaml_compiler_flags = ctx.attrs.ocaml_compiler_flags,
            yacc_compiler = ctx.attrs.yacc_compiler[RunInfo],
        ),
        OCamlPlatformInfo(
            name = ctx.attrs.name,
        ),
    ]

_config_backed_ocaml_toolchain_rule = rule(
    impl = _config_backed_ocaml_toolchain_rule_impl,
    attrs = {
        "binutils_ld": attrs.dep(default = "fbcode//third-party-buck/platform010/build/binutils:bin/ld"),
        "debug": attrs.dep(providers = [RunInfo]),
        "dep_tool": attrs.dep(providers = [RunInfo]),
        "interop_includes": attrs.source(),
        "lex_compiler": attrs.dep(providers = [RunInfo]),
        "libasmrun": attrs.source(default = "fbcode//third-party-buck/platform010/build/supercaml:libasmrun.a"),
        "ocaml_bytecode_compiler": attrs.dep(providers = [RunInfo]),
        "ocaml_compiler": attrs.dep(providers = [RunInfo]),
        "ocaml_compiler_flags": attrs.list(attrs.string()),
        "warnings_flags": attrs.string(),  # must be a single argument
        "yacc_compiler": attrs.dep(providers = [RunInfo]),
    },
)
