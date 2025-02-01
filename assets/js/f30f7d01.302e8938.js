"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[2660],{59921:(e,s,l)=>{l.r(s),l.d(s,{assets:()=>o,contentTitle:()=>a,default:()=>h,frontMatter:()=>i,metadata:()=>d,toc:()=>c});var r=l(74848),n=l(28453),t=l(28774);const i={},a="cli_args",d={id:"api/bxl/cli_args",title:"cli_args",description:"bool",source:"@site/../docs/api/bxl/cli_args.md",sourceDirName:"api/bxl",slug:"/api/bxl/cli_args",permalink:"/docs/api/bxl/cli_args",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{},sidebar:"apiSidebar",previous:{title:"UqueryContext",permalink:"/docs/api/bxl/UqueryContext"}},o={},c=[{value:"bool",id:"bool",level:2},{value:"enum",id:"enum",level:2},{value:"float",id:"float",level:2},{value:"int",id:"int",level:2},{value:"json",id:"json",level:2},{value:"list",id:"list",level:2},{value:"option",id:"option",level:2},{value:"string",id:"string",level:2},{value:"sub_target",id:"sub_target",level:2},{value:"sub_target_expr",id:"sub_target_expr",level:2},{value:"target_expr",id:"target_expr",level:2},{value:"target_label",id:"target_label",level:2}];function x(e){const s={h1:"h1",h2:"h2",header:"header",hr:"hr",...(0,n.R)(),...e.components};return(0,r.jsxs)(r.Fragment,{children:[(0,r.jsx)(s.header,{children:(0,r.jsx)(s.h1,{id:"cli_args",children:"cli_args"})}),"\n",(0,r.jsx)(s.h2,{id:"bool",children:"bool"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def bool(\ndefault = False,\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"enum",children:"enum"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def enum(\nvariants: list[",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),"] | tuple[",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),", ...],\n/,\ndefault = ...,\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"float",children:"float"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def float(\ndefault = ...,\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"int",children:"int"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def int(\ndefault = ...,\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"json",children:"json"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def json(\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"list",children:"list"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def list(\ninner: ",(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"}),",\n/,\ndefault = ...,\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"option",children:"option"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def option(\ninner: ",(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"}),",\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\ndefault = None,\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"string",children:"string"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def string(\ndefault = ...,\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"sub_target",children:"sub_target"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def sub_target(\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"sub_target_expr",children:"sub_target_expr"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def sub_target_expr(\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"target_expr",children:"target_expr"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def target_expr(\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})}),"\n",(0,r.jsx)(s.hr,{}),"\n",(0,r.jsx)(s.h2,{id:"target_label",children:"target_label"}),"\n",(0,r.jsx)("pre",{class:"language-python",children:(0,r.jsxs)("code",{children:["def target_label(\ndoc: ",(0,r.jsx)(t.default,{to:"/docs/api/starlark/str",children:"str"}),' = "",\n*,\nshort = ...,\n) -> ',(0,r.jsx)(t.default,{to:"/docs/api/bxl/CliArgs",children:"bxl.CliArgs"})]})})]})}function h(e={}){const{wrapper:s}={...(0,n.R)(),...e.components};return s?(0,r.jsx)(s,{...e,children:(0,r.jsx)(x,{...e})}):x(e)}},28453:(e,s,l)=>{l.d(s,{R:()=>i,x:()=>a});var r=l(96540);const n={},t=r.createContext(n);function i(e){const s=r.useContext(t);return r.useMemo((function(){return"function"==typeof e?e(s):{...s,...e}}),[s,e])}function a(e){let s;return s=e.disableParentContext?"function"==typeof e.components?e.components(n):e.components||n:i(e.components),r.createElement(t.Provider,{value:s},e.children)}}}]);