def suffix_symbols(
        ctx: "context",
        suffix: str.type,
        objects: ["artifact"],
        cxx_toolchain: "CxxToolchainInfo") -> ["artifact"]:
    """
    Take a list of objects and append a suffix to all  defined symbols.
    The script does the following:
    """
    objcopy = cxx_toolchain.binary_utilities_info.objcopy
    nm = cxx_toolchain.binary_utilities_info.nm
    artifacts = []
    for original in objects:
        artifact = ctx.actions.declare_output(suffix + "_" + original.basename)

        script_env = {
            "NM": nm,
            "OBJCOPY": objcopy,
            "ORIGINAL": original,
            "OUT": artifact.as_output(),
        }

        script = (
            "set -euo pipefail; " +  # fail if any command in the script fails
            "export SYMSFILE=$(mktemp);" +  # create a temporary file
            '"$NM" --defined-only "$ORIGINAL" | ' +  # print all defined symbols in the original object
            # the output from nm looks like this: '0000000000000000 T PyInit_hello'
            ' awk \'{{print $3" "$3"{suffix}"}}\' > '.format(suffix = "_" + suffix) +  # using awk we print the symbol name 'PyInit_hello' followed by the symbol name with the suffix appended
            '"$SYMSFILE";' +  # we compile a file with a line for each symbol containing the 'symbol_name symbol_name{suffix}'
            '"$OBJCOPY" --redefine-syms="$SYMSFILE" "$ORIGINAL" "$OUT"'  # using objcopy we pass in the symbols file to re-write the original symbol name to the now suffixed version
        )

        # Usage: objcopy [option(s)] in-file [out-file]
        ctx.actions.run(
            [
                "/bin/bash",
                "-c",
                script,
            ],
            env = script_env,
            category = "suffix_symbols",
            identifier = artifact.basename,
        )
        artifacts.append(artifact)
    return artifacts
