load("@prelude//python:python.bzl", "PythonLibraryInfo")
load("@prelude//utils:utils.bzl", "expect", "map_idx")
load(
    ":link_info.bzl",
    "LinkInfo",  # @unused Used as a type
    "LinkInfos",
    "LinkStyle",
    "Linkage",
    "LinkedObject",
    "MergedLinkInfo",
    "get_actual_link_style",
    "get_link_styles_for_linkage",
    _get_link_info = "get_link_info",
)

# A provider with information used to link a rule into a shared library.
# Potential omnibus roots must provide this so that omnibus can link them
# here, in the context of the top-level packaging rule.
NativeLinkTargetInfo = provider(fields = [
    "link_infos",  # LinkInfos.type
    "name",  # [str.type, None]
    "deps",  # ["label"]
    "shared_root",  # SharedOmnibusRoot.type
])

###############################################################################
# Linkable Graph collects information on a node in the target graph that
# contains linkable output. This graph information may then be provided to any
# consumers of this target.
###############################################################################

LinkableNode = record(
    # Attribute labels on the target.
    labels = field([str.type], []),
    # Prefered linkage for this target.
    preferred_linkage = field([Linkage.type, None]),
    # Linkable deps of this target.
    deps = field(["label"], []),
    # Exported linkable deps of this target.
    #
    # We distinguish between deps and exported deps so that when creating shared
    # libraries in a large graph we only need to link each library against its
    # deps and their (transitive) exported deps. This helps keep link lines smaller
    # and produces more efficient libs (for example, DT_NEEDED stays a manageable size).
    exported_deps = field(["label"], []),
    # Link infos for all supported link styles.
    link_infos = field({LinkStyle.type: LinkInfos.type}, {}),
    # Shared libraries provided by this target.  Used if this target is
    # excluded.
    shared_libs = field({str.type: LinkedObject.type}, {}),
)

LinkableGraphNode = record(
    # Target/label of this node
    label = field("label"),

    # If this node has linkable output, it's linkable data
    linkable = field([LinkableNode.type, None], None),

    # All potential root notes for an omnibus link (e.g. C++ libraries,
    # C++ Python extensions).
    roots = field({"label": NativeLinkTargetInfo.type}, {}),

    # Exclusions this node adds to the Omnibus graph
    excluded = field({"label": None}, {}),
)

LinkableGraphTSet = transitive_set()

# The LinkableGraph for a target holds all the transitive nodes, roots, and exclusions
# from all of its dependencies.
#
# TODO(cjhopman): Rather than flattening this at each node, we should build up an actual
# graph structure.
LinkableGraph = provider(fields = [
    # Target identifier of the graph.
    "label",  # "label"
    "nodes",  # "LinkableGraphTSet"
])

def create_linkable_node(
        ctx: "context",
        preferred_linkage: [Linkage.type, None] = None,
        deps: ["dependency"] = [],
        exported_deps: ["dependency"] = [],
        link_infos: {LinkStyle.type: LinkInfos.type} = {},
        shared_libs: {str.type: LinkedObject.type} = {}) -> LinkableNode.type:
    if preferred_linkage:
        for link_style in get_link_styles_for_linkage(preferred_linkage):
            expect(
                link_style in link_infos,
                "must have {} link info".format(link_style),
            )

    return LinkableNode(
        labels = ctx.attrs.labels,
        preferred_linkage = preferred_linkage,
        deps = linkable_deps(deps),
        exported_deps = linkable_deps(exported_deps),
        link_infos = link_infos,
        shared_libs = shared_libs,
    )

def create_linkable_graph_node(
        ctx: "context",
        linkable_node: [LinkableNode.type, None] = None,
        roots: {"label": NativeLinkTargetInfo.type} = {},
        excluded: {"label": None} = {}) -> LinkableGraphNode.type:
    return LinkableGraphNode(
        label = ctx.label,
        linkable = linkable_node,
        roots = roots,
        excluded = excluded,
    )

def create_linkable_graph(
        ctx: "context",
        node: [LinkableGraphNode.type, None] = None,
        deps: ["dependency"] = [],
        children: [LinkableGraph.type] = []) -> LinkableGraph.type:
    all_children_graphs = filter(None, map_idx(LinkableGraph, deps)) + children
    kwargs = {
        "children": [child_node.nodes for child_node in all_children_graphs],
    }
    if node:
        kwargs["value"] = node
    return LinkableGraph(
        label = ctx.label,
        nodes = ctx.actions.tset(LinkableGraphTSet, **kwargs),
    )

def get_linkable_graph_node_map_func(graph: LinkableGraph.type):
    def get_linkable_graph_node_map() -> {"label": LinkableNode.type}:
        nodes = graph.nodes.traverse()
        linkable_nodes = {}
        for node in filter(None, nodes):
            if node.linkable:
                linkable_nodes[node.label] = node.linkable
        return linkable_nodes

    return get_linkable_graph_node_map

def linkable_deps(deps: ["dependency"]) -> ["label"]:
    labels = []

    for dep in deps:
        dep_info = linkable_graph(dep)
        if dep_info != None:
            labels.append(dep_info.label)

    return labels

def linkable_graph(dep: "dependency") -> [LinkableGraph.type, None]:
    """
    Helper to extract `LinkableGraph` from a dependency which also
    provides `MergedLinkInfo`.
    """

    # We only care about "linkable" deps.
    if dep[PythonLibraryInfo] != None or dep[MergedLinkInfo] == None:
        return None

    expect(
        dep[LinkableGraph] != None,
        "{} provides `MergedLinkInfo`".format(dep.label) +
        " but doesn't also provide `LinkableGraph`",
    )

    return dep[LinkableGraph]

def get_link_info(
        node: LinkableNode.type,
        link_style: LinkStyle.type,
        prefer_stripped: bool.type = False,
        force_no_link_groups = False) -> LinkInfo.type:
    info = _get_link_info(
        node.link_infos[link_style],
        prefer_stripped = prefer_stripped,
    )

    if force_no_link_groups and info.use_link_groups:
        return LinkInfo(
            name = info.name,
            pre_flags = info.pre_flags,
            post_flags = info.post_flags,
            linkables = info.linkables,
            use_link_groups = False,
        )
    return info

def get_deps_for_link(
        node: LinkableNode.type,
        link_style: LinkStyle.type) -> ["label"]:
    """
    Return deps to follow when linking against this node with the given link
    style.
    """

    # Avoid making a copy of the list until we know have to modify it.
    deps = node.exported_deps

    # If we're linking statically, include non-exported deps.
    actual = get_actual_link_style(link_style, node.preferred_linkage)
    if actual != LinkStyle("shared") and node.deps:
        # Important that we don't mutate deps, but create a new list
        deps = deps + node.deps

    return deps
