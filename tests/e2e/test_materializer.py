import sys

from xplat.build_infra.buck_e2e.api.buck import Buck
from xplat.build_infra.buck_e2e.buck_workspace import buck_test, env


"""
If you need to add a directory that's isolated in buck2/test/targets
(ex. some test of form @buck_test(inplace=False, data_dir=some_new_directory)),
then you will need to update isolated_targets in buck2/test/targets/TARGETS.
Otherwise the test will fail because it cannot recognize the new directory.
"""

# Eden materializer only available on Linux
def eden_linux_only() -> bool:
    return sys.platform == "linux"


def watchman_dependency_linux_only() -> bool:
    return sys.platform == "linux"


@buck_test(inplace=False, data_dir="modify_deferred_materialization")
async def test_modify_input_source(buck: Buck) -> None:
    await buck.build("//:urandom_dep")

    targets_file = buck.cwd / "TARGETS.fixture"

    # Change the label in Targets.
    with open(targets_file, encoding="utf-8") as f:
        targets = f.read()

    targets = targets.replace("__NOT_A_REAL_LABEL__", "buck2_test_local_exec")

    with open(targets_file, "w", encoding="utf-8") as f:
        f.write(targets)

    await buck.build("//:urandom_dep")


@buck_test(inplace=False, data_dir="modify_deferred_materialization_deps")
async def test_modify_dep_materialization(buck: Buck) -> None:
    await buck.build("//:check")

    with open(buck.cwd / "text", "w", encoding="utf-8") as f:
        f.write("TEXT2")

    await buck.build("//:check")


@buck_test(inplace=False, data_dir="modify_deferred_materialization_deps")
@env("BUCK_LOG", "buck2_build_api::execute::materializer=trace")
async def test_local_caching_of_re_artifacts_on_deferred_materializer(
    buck: Buck,
) -> None:
    target = "root//:remote_text"
    result = await buck.build(target)
    # Check output is correctly materialized
    assert result.get_build_report().output_for_target(target).exists()

    # In this case, modifying the input does not change the output, so the output should not
    # need to be rematerialized
    with open(buck.cwd / "text", "w", encoding="utf-8") as f:
        f.write("TEXT2")

    result = await buck.build(target)
    # Check output still exists
    assert result.get_build_report().output_for_target(target).exists()
    # Check that materializer did not report any rematerialization
    assert "already materialized, no need to declare again" in result.stderr
    assert "materialize artifact" not in result.stderr


@buck_test(inplace=False, data_dir="modify_deferred_materialization_deps")
@env("BUCK_LOG", "buck2_build_api::execute::materializer=trace")
async def test_local_caching_of_re_artifacts_on_deferred_materializer_disabled_without_buckconfig(
    buck: Buck,
) -> None:
    # Disable local caching of RE artifacts
    buckconfig_file = buck.cwd / ".buckconfig"
    with open(buckconfig_file, encoding="utf-8") as f:
        buckconfig = f.read()
    buckconfig = buckconfig.replace(
        "enable_local_caching_of_re_artifacts = true",
        "enable_local_caching_of_re_artifacts = false",
    )
    with open(buckconfig_file, "w", encoding="utf-8") as f:
        f.write(buckconfig)

    target = "root//:remote_text"
    result = await buck.build(target)
    # Check output is correctly materialized
    assert result.get_build_report().output_for_target(target).exists()

    with open(buck.cwd / "text", "w", encoding="utf-8") as f:
        f.write("TEXT2")

    result = await buck.build(target)
    # Check output still exists
    assert result.get_build_report().output_for_target(target).exists()
    # Check that materializer did have to rematerialize in this case
    assert "already materialized, no need to declare again" not in result.stderr
    assert "materialize artifact" in result.stderr


if eden_linux_only():

    @buck_test(inplace=False, data_dir="eden_materializer")
    async def test_eden_materialization_simple(buck: Buck) -> None:
        await buck.build("//:simple")


def set_materializer(buck: Buck, old: str, new: str) -> None:
    config_file = buck.cwd / ".buckconfig"

    # Change the label in Targets.
    with open(config_file, encoding="utf-8") as f:
        config = f.read()
    old_config = "materializations = {}".format(old)
    new_config = "materializations = {}".format(new)
    config = config.replace(old_config, new_config)

    with open(config_file, "w", encoding="utf-8") as f:
        f.write(config)


if eden_linux_only():

    @buck_test(inplace=False, data_dir="eden_materializer")
    async def test_eden_materialization_clean_after_config_change(buck: Buck) -> None:
        set_materializer(buck, "eden", "deferred")
        await buck.build("//:simple")

        set_materializer(buck, "deferred", "eden")
        await buck.kill()
        await buck.build("//:simple")


if eden_linux_only():

    @buck_test(inplace=False, data_dir="eden_materializer")
    async def test_eden_materialization_no_config_change(buck: Buck) -> None:
        await buck.build("//:simple")
        await buck.kill()
        await buck.build("//:simple")
