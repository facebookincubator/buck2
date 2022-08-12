load("@fbcode//buck2/prelude:attributes.bzl", "Linkage", "Traversal")
load(
    "@fbcode//buck2/prelude/cxx:groups.bzl",
    "compute_mappings",
    "parse_groups_definitions",
)
load(
    "@fbcode//buck2/prelude/cxx:link_groups.bzl",
    "LinkGroupInfo",
)
load(
    "@fbcode//buck2/prelude/linking:linkable_graph.bzl",
    "create_merged_linkable_graph",
)
load("@fbcode//buck2/prelude/user:rule_spec.bzl", "RuleRegistrationSpec")

def _v1_attrs():
    return attrs.list(attrs.tuple(attrs.string(), attrs.list(attrs.tuple(attrs.dep(), attrs.enum(Traversal), attrs.option(attrs.string()), attrs.option(attrs.enum(Linkage))))))

def link_group_map_attr():
    v2_attrs = attrs.dep(providers = [LinkGroupInfo])
    return attrs.option(attrs.one_of(v2_attrs, _v1_attrs()), default = None)

def _impl(ctx: "context") -> ["provider"]:
    link_groups = parse_groups_definitions(ctx.attrs.map)
    link_group_deps = [mapping.target for group in link_groups for mapping in group.mappings]
    linkable_graph = create_merged_linkable_graph(
        ctx.label,
        link_group_deps,
    )
    mappings = compute_mappings(groups = link_groups, graph = linkable_graph)
    return [
        DefaultInfo(),
        LinkGroupInfo(groups_hash = hash(str(link_groups)), mappings = mappings),
    ]

registration_spec = RuleRegistrationSpec(
    name = "link_group_map",
    impl = _impl,
    attrs = {
        "map": _v1_attrs(),
    },
)
