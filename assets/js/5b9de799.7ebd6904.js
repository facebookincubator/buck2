"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[3691],{53232:(e,r,t)=>{t.r(r),t.d(r,{assets:()=>o,contentTitle:()=>d,default:()=>u,frontMatter:()=>i,metadata:()=>s,toc:()=>g});var a=t(74848),n=t(28453),l=t(28774);const i={},d="ConfiguredTargetLabel",s={id:"api/build/ConfiguredTargetLabel",title:"ConfiguredTargetLabel",description:"ConfiguredTargetLabel.cell",source:"@site/../docs/api/build/ConfiguredTargetLabel.md",sourceDirName:"api/build",slug:"/api/build/ConfiguredTargetLabel",permalink:"/docs/api/build/ConfiguredTargetLabel",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{},sidebar:"apiSidebar",previous:{title:"ConfiguredProvidersLabel",permalink:"/docs/api/build/ConfiguredProvidersLabel"},next:{title:"ConstraintSettingInfo",permalink:"/docs/api/build/ConstraintSettingInfo"}},o={},g=[{value:"ConfiguredTargetLabel.cell",id:"configuredtargetlabelcell",level:2},{value:"ConfiguredTargetLabel.config",id:"configuredtargetlabelconfig",level:2},{value:"ConfiguredTargetLabel.name",id:"configuredtargetlabelname",level:2},{value:"ConfiguredTargetLabel.package",id:"configuredtargetlabelpackage",level:2},{value:"ConfiguredTargetLabel.path",id:"configuredtargetlabelpath",level:2},{value:"ConfiguredTargetLabel.raw_target",id:"configuredtargetlabelraw_target",level:2},{value:"ConfiguredTargetLabel.with_sub_target",id:"configuredtargetlabelwith_sub_target",level:2}];function c(e){const r={code:"code",h1:"h1",h2:"h2",header:"header",hr:"hr",p:"p",pre:"pre",...(0,n.R)(),...e.components};return(0,a.jsxs)(a.Fragment,{children:[(0,a.jsx)(r.header,{children:(0,a.jsx)(r.h1,{id:"configuredtargetlabel",children:"ConfiguredTargetLabel"})}),"\n",(0,a.jsx)(r.h2,{id:"configuredtargetlabelcell",children:"ConfiguredTargetLabel.cell"}),"\n",(0,a.jsx)("pre",{class:"language-python",children:(0,a.jsxs)("code",{children:["ConfiguredTargetLabel.cell: ",(0,a.jsx)(l.default,{to:"/docs/api/starlark/str",children:"str"})]})}),"\n",(0,a.jsx)(r.hr,{}),"\n",(0,a.jsx)(r.h2,{id:"configuredtargetlabelconfig",children:"ConfiguredTargetLabel.config"}),"\n",(0,a.jsx)("pre",{class:"language-python",children:(0,a.jsx)("code",{children:"def ConfiguredTargetLabel.config() -> configuration"})}),"\n",(0,a.jsx)(r.hr,{}),"\n",(0,a.jsx)(r.h2,{id:"configuredtargetlabelname",children:"ConfiguredTargetLabel.name"}),"\n",(0,a.jsx)("pre",{class:"language-python",children:(0,a.jsxs)("code",{children:["ConfiguredTargetLabel.name: ",(0,a.jsx)(l.default,{to:"/docs/api/starlark/str",children:"str"})]})}),"\n",(0,a.jsx)(r.hr,{}),"\n",(0,a.jsx)(r.h2,{id:"configuredtargetlabelpackage",children:"ConfiguredTargetLabel.package"}),"\n",(0,a.jsx)("pre",{class:"language-python",children:(0,a.jsxs)("code",{children:["ConfiguredTargetLabel.package: ",(0,a.jsx)(l.default,{to:"/docs/api/starlark/str",children:"str"})]})}),"\n",(0,a.jsx)(r.hr,{}),"\n",(0,a.jsx)(r.h2,{id:"configuredtargetlabelpath",children:"ConfiguredTargetLabel.path"}),"\n",(0,a.jsx)("pre",{class:"language-python",children:(0,a.jsxs)("code",{children:["ConfiguredTargetLabel.path: ",(0,a.jsx)(l.default,{to:"/docs/api/build/CellPath",children:"CellPath"})]})}),"\n",(0,a.jsx)(r.hr,{}),"\n",(0,a.jsx)(r.h2,{id:"configuredtargetlabelraw_target",children:"ConfiguredTargetLabel.raw_target"}),"\n",(0,a.jsx)("pre",{class:"language-python",children:(0,a.jsxs)("code",{children:["def ConfiguredTargetLabel.raw_target(\n) -> ",(0,a.jsx)(l.default,{to:"/docs/api/build/TargetLabel",children:"target_label"})]})}),"\n",(0,a.jsx)(r.p,{children:"Returns the unconfigured underlying target label."}),"\n",(0,a.jsx)(r.hr,{}),"\n",(0,a.jsx)(r.h2,{id:"configuredtargetlabelwith_sub_target",children:"ConfiguredTargetLabel.with_sub_target"}),"\n",(0,a.jsx)("pre",{class:"language-python",children:(0,a.jsxs)("code",{children:["def ConfiguredTargetLabel.with_sub_target(\nsubtarget_name: ",(0,a.jsx)(l.default,{to:"/docs/api/starlark/str",children:"str"})," | list[",(0,a.jsx)(l.default,{to:"/docs/api/starlark/str",children:"str"}),"] = ...,\n) -> ",(0,a.jsx)(l.default,{to:"/docs/api/build/Label",children:"label"})]})}),"\n",(0,a.jsxs)(r.p,{children:["Converts a ",(0,a.jsx)(r.code,{children:"ConfiguredTargetLabel"})," into its corresponding ",(0,a.jsx)(r.code,{children:"Label"})," given the subtarget name which is a list for each layer of subtarget"]}),"\n",(0,a.jsx)(r.p,{children:"Sample usage:"}),"\n",(0,a.jsx)(r.pre,{children:(0,a.jsx)(r.code,{className:"language-text",children:'def _impl_sub_target(ctx):\n    owners = ctx.cquery().owner("bin/TARGETS.fixture")\n    for owner in owners:\n        configured_label = owner.label\n        ctx.output.print(configured_label.with_sub_target())\n        ctx.output.print(configured_label.with_sub_target("subtarget1"))\n        ctx.output.print(configured_label.with_sub_target(["subtarget1", "subtarget2"]))\n'})})]})}function u(e={}){const{wrapper:r}={...(0,n.R)(),...e.components};return r?(0,a.jsx)(r,{...e,children:(0,a.jsx)(c,{...e})}):c(e)}},28453:(e,r,t)=>{t.d(r,{R:()=>i,x:()=>d});var a=t(96540);const n={},l=a.createContext(n);function i(e){const r=a.useContext(l);return a.useMemo((function(){return"function"==typeof e?e(r):{...r,...e}}),[r,e])}function d(e){let r;return r=e.disableParentContext?"function"==typeof e.components?e.components(n):e.components||n:i(e.components),a.createElement(l.Provider,{value:r},e.children)}}}]);