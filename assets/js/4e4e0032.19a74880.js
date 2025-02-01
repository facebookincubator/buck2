"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[2824],{48209:(e,t,a)=>{a.r(t),a.d(t,{assets:()=>i,contentTitle:()=>c,default:()=>p,frontMatter:()=>s,metadata:()=>o,toc:()=>d});var n=a(74848),r=a(28453),l=a(28774);const s={},c="Lazy",o={id:"api/bxl/Lazy",title:"Lazy",description:"bxl.Lazy can be resolved to the actual result. The computation only happens when called .resolve() or .catch().resolve().",source:"@site/../docs/api/bxl/Lazy.md",sourceDirName:"api/bxl",slug:"/api/bxl/Lazy",permalink:"/docs/api/bxl/Lazy",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{},sidebar:"apiSidebar",previous:{title:"Filesystem",permalink:"/docs/api/bxl/Filesystem"},next:{title:"LazyAttrs",permalink:"/docs/api/bxl/LazyAttrs"}},i={},d=[{value:"Lazy.catch",id:"lazycatch",level:2},{value:"Lazy.resolve",id:"lazyresolve",level:2}];function h(e){const t={a:"a",code:"code",h1:"h1",h2:"h2",header:"header",hr:"hr",p:"p",pre:"pre",...(0,r.R)(),...e.components};return(0,n.jsxs)(n.Fragment,{children:[(0,n.jsx)(t.header,{children:(0,n.jsx)(t.h1,{id:"lazy",children:"Lazy"})}),"\n",(0,n.jsxs)(t.p,{children:["bxl.Lazy can be resolved to the actual result. The computation only happens when called ",(0,n.jsx)(t.code,{children:".resolve()"})," or ",(0,n.jsx)(t.code,{children:".catch().resolve()"}),"."]}),"\n",(0,n.jsx)(t.h2,{id:"lazycatch",children:"Lazy.catch"}),"\n",(0,n.jsx)("pre",{class:"language-python",children:(0,n.jsxs)("code",{children:["def Lazy.catch() -> ",(0,n.jsx)(l.default,{to:"/docs/api/bxl/Lazy",children:"bxl.Lazy"})]})}),"\n",(0,n.jsxs)(t.p,{children:["Make ",(0,n.jsx)(t.code,{children:"Lazy"})," can be resolved later by catching the error."]}),"\n",(0,n.jsx)(t.p,{children:"Example:"}),"\n",(0,n.jsx)(t.pre,{children:(0,n.jsx)(t.code,{className:"language-python",children:'def _impl(ctx):\n    target = ctx.configured_targets("cell//path/to:target")\n    analysis_result = ctx.lazy.analysis(target).catch().resolve()\n'})}),"\n",(0,n.jsx)(t.hr,{}),"\n",(0,n.jsx)(t.h2,{id:"lazyresolve",children:"Lazy.resolve"}),"\n",(0,n.jsx)("pre",{class:"language-python",children:(0,n.jsx)("code",{children:"def Lazy.resolve()"})}),"\n",(0,n.jsxs)(t.p,{children:["Resolve the operation to the final result. When called via ",(0,n.jsx)(t.code,{children:".catch().resolve()"}),", the error will be catched and returned as a ",(0,n.jsx)(t.a,{href:"../Result",children:(0,n.jsx)(t.code,{children:"bxl.Result"})}),". Otherwise, it will return the raw type without catching the error."]}),"\n",(0,n.jsx)(t.p,{children:"Example:"}),"\n",(0,n.jsx)(t.pre,{children:(0,n.jsx)(t.code,{className:"language-python",children:'def _impl(ctx):\n    target = ctx.configured_targets("cell//path/to:target")\n    analysis_result = ctx.lazy.analysis(target).resolve()\n'})})]})}function p(e={}){const{wrapper:t}={...(0,r.R)(),...e.components};return t?(0,n.jsx)(t,{...e,children:(0,n.jsx)(h,{...e})}):h(e)}},28453:(e,t,a)=>{a.d(t,{R:()=>s,x:()=>c});var n=a(96540);const r={},l=n.createContext(r);function s(e){const t=n.useContext(l);return n.useMemo((function(){return"function"==typeof e?e(t):{...t,...e}}),[t,e])}function c(e){let t;return t=e.disableParentContext?"function"==typeof e.components?e.components(r):e.components||r:s(e.components),n.createElement(l.Provider,{value:t},e.children)}}}]);