"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[4380],{66709:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>o,contentTitle:()=>c,default:()=>d,frontMatter:()=>l,metadata:()=>a,toc:()=>r});var i=t(74848),s=t(28453);const l={id:"buck1_vs_buck2",title:"Buck1 vs Buck2"},c=void 0,a={id:"developers/architecture/buck1_vs_buck2",title:"Buck1 vs Buck2",description:"At a glance",source:"@site/../docs/developers/architecture/buck1_vs_buck2.md",sourceDirName:"developers/architecture",slug:"/developers/architecture/buck1_vs_buck2",permalink:"/docs/developers/architecture/buck1_vs_buck2",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{id:"buck1_vs_buck2",title:"Buck1 vs Buck2"},sidebar:"main",previous:{title:"Architectural Model",permalink:"/docs/developers/architecture/buck2"},next:{title:"Finding Commands That Buck2 Ran",permalink:"/docs/developers/what-ran"}},o={},r=[{value:"At a glance",id:"at-a-glance",level:2},{value:"Top-down vs Bottom-up - understanding the implications of the difference in execution models between Buck1 and Buck2",id:"top-down-vs-bottom-up---understanding-the-implications-of-the-difference-in-execution-models-between-buck1-and-buck2",level:2},{value:"What are the differences?",id:"what-are-the-differences",level:3},{value:"Building A with Buck1",id:"building-a-with-buck1",level:4},{value:"Building A with Buck2",id:"building-a-with-buck2",level:4},{value:"Some implications",id:"some-implications",level:3},{value:"Rulekeys vs Action digests",id:"rulekeys-vs-action-digests",level:4},{value:"Buck2 queries many more caches",id:"buck2-queries-many-more-caches",level:4},{value:"Materialization",id:"materialization",level:4},{value:"Second-order implications",id:"second-order-implications",level:3},{value:"Non-determinism",id:"non-determinism",level:4},{value:"Cache misses don\u2019t necessarily propagate",id:"cache-misses-dont-necessarily-propagate",level:4},{value:"Hybrid execution",id:"hybrid-execution",level:4}];function h(e){const n={a:"a",code:"code",em:"em",h2:"h2",h3:"h3",h4:"h4",li:"li",p:"p",strong:"strong",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...(0,s.R)(),...e.components};return(0,i.jsxs)(i.Fragment,{children:[(0,i.jsx)(n.h2,{id:"at-a-glance",children:"At a glance"}),"\n",(0,i.jsx)(n.p,{children:"The following table provides an at-a-glance comparison of Buck1 and Buck2."}),"\n",(0,i.jsxs)(n.table,{children:[(0,i.jsx)(n.thead,{children:(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.th,{style:{textAlign:"left"},children:"Buck1"}),(0,i.jsx)(n.th,{style:{textAlign:"left"},children:"Buck2"})]})}),(0,i.jsxs)(n.tbody,{children:[(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Build files in Starlark"}),(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Build files in Starlark"})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Macros in Starlark"}),(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Macros in Starlark"})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Rules in Java"}),(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Rules in Starlark"})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Rules and Macros are logically similar"}),(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Rules and Macros are logically similar"})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Rules and Core are not well abstracted"}),(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Rules and Core are strongly separated"})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Core in Java"}),(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Core in Rust"})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Remote Execution (RE) not well supported"}),(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"All rules support remote execution by default"})]}),(0,i.jsxs)(n.tr,{children:[(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Varying degrees of incrementality / parallelism"}),(0,i.jsx)(n.td,{style:{textAlign:"left"},children:"Unified incrementality / parallelism"})]})]})]}),"\n",(0,i.jsx)(n.h2,{id:"top-down-vs-bottom-up---understanding-the-implications-of-the-difference-in-execution-models-between-buck1-and-buck2",children:"Top-down vs Bottom-up - understanding the implications of the difference in execution models between Buck1 and Buck2"}),"\n",(0,i.jsx)(n.p,{children:"It is often said that Buck1 does 'top down' and Buck2 does 'bottom up' building.\nThis results in cases where some topics that seem conceptually trivial in Buck1\nare hard problems in Buck2, or vice versa."}),"\n",(0,i.jsx)(n.h3,{id:"what-are-the-differences",children:"What are the differences?"}),"\n",(0,i.jsxs)(n.p,{children:[(0,i.jsx)(n.strong,{children:"Scenario"}),": Imagine you are building A, which depends on both B and C, but\nwhere neither B nor C have any dependencies."]}),"\n",(0,i.jsx)(n.p,{children:"For the sake of simplicity, imagine B and C are C++ compilations (that produce\nobject files), and A is a link (that consumes them and produces a shared\nlibrary)."}),"\n",(0,i.jsx)(n.h4,{id:"building-a-with-buck1",children:"Building A with Buck1"}),"\n",(0,i.jsx)(n.p,{children:"Following is an oversimplified view of what happens:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["Buck1 computes the 'rulekey' for B.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"This consists of mixing together the hashes of the C++ file being compiled,\nas well as all C++ compiler flags, and so on."}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.li,{children:"Buck1 then does the same for C."}),"\n",(0,i.jsxs)(n.li,{children:["Buck1 then computes the rulekey for A.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"This consist of mixing together the rulekeys of B and C, as well as linker\nflags used by A. for example."}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:["Buck1 then looks up the rulekey for A in the cache.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"If there's a hit, then Buck1 downloads the output and its job done."}),"\n",(0,i.jsx)(n.li,{children:"If there's a cache miss, continue."}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:["Buck1 then queries the cache for the rulekeys of B and C:","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"If there's a hit, then the output is downloaded."}),"\n",(0,i.jsx)(n.li,{children:"If there's a miss, then Buck1 runs the commands needed to produce the object\nfile that was missed. Regardless of whether those commands run locally or on\nRE, Buck1 downloads the output of B and C."}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:["Buck1 then runs the command for A to produce the shared library.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["At this point, Buck1 may actually do another cache lookup with a different\nrulekey, which is called an ",(0,i.jsx)(n.em,{children:"input based rulekey"}),". This rulekey is derived\nfrom the inputs of the action that needs executing, which at this point of\nthe build are known (since they were just built)!"]}),"\n"]}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.h4,{id:"building-a-with-buck2",children:"Building A with Buck2"}),"\n",(0,i.jsx)(n.p,{children:"In contrast, if you ask Buck2 to build A, here is what happens:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsxs)(n.li,{children:["Buck2 produce the action to compile B and computes the hash of the action.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"This is the 'action digest', which consists of mixing the hashes of all the\ninputs (such as the C++ file), as well as the command line (so, implicitly,\nthe compiler flags)."}),"\n"]}),"\n"]}),"\n",(0,i.jsxs)(n.li,{children:["Buck2 queries the action cache for the action digest hash.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"If there's a hit, Buck2 obtains the hash of the resulting object file (that\nis, the output of B)."}),"\n",(0,i.jsx)(n.li,{children:"If there's a miss, Buck2 runs the action on RE (or potentially locally) and\nobtains the hash of the object file. If the action runs remotely, Buck2 will\nnot download the output."}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.li,{children:"Buck2 does the same thing for C."}),"\n",(0,i.jsxs)(n.li,{children:["Buck2 produces the action to link A.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"This consists of mixing together all the hashes of the input files (which\nwere retrieved earlier) and the command line to produce an action digest,\nthen querying the cache and potentially running the action."}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.li,{children:"Once Buck2 produces A (again, on RE), then, since this output was requested by\nthe user (unlike the intermediary outputs B and C), Buck2 downloads A."}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"some-implications",children:"Some implications"}),"\n",(0,i.jsx)(n.h4,{id:"rulekeys-vs-action-digests",children:"Rulekeys vs Action digests"}),"\n",(0,i.jsx)(n.p,{children:"The closest thing to Buck1\u2019s rulekey in Buck2 is the action digest, but they are\nvery different!"}),"\n",(0,i.jsx)(n.p,{children:"Since it\u2019s a product of the (transitive) inputs of an action, the (default)\nrulekey can be computed without running anything or querying any caches.\nHowever, the action digest cannot: it requires the actual inputs of an action,\nwhich means you need to build all the dependencies first."}),"\n",(0,i.jsx)(n.p,{children:"This means that:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"In Buck1, you can ask for rulekeys for a target."}),"\n",(0,i.jsxs)(n.li,{children:["In Buck2, you\u2019d have to run the build first then ask for the action digests\n(this is what the ",(0,i.jsx)(n.code,{children:"buck2 log what-ran"})," would show you)."]}),"\n"]}),"\n",(0,i.jsx)(n.h4,{id:"buck2-queries-many-more-caches",children:"Buck2 queries many more caches"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"Buck1 will not descend further down a tree of dependency when it gets a cache\nhit."}),"\n",(0,i.jsx)(n.li,{children:"Buck2 will always walk up all your dependencies, regardless of whether you get\ncache hits or not."}),"\n"]}),"\n",(0,i.jsx)(n.h4,{id:"materialization",children:"Materialization"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"When Buck1 gets a cache miss, it downloads the outputs."}),"\n",(0,i.jsxs)(n.li,{children:["Buck2, by contract, does not download outputs as part of a build (this is\ncalled 'deferred materialization').","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"Note that Buck2 does download the outputs if the user asked for them (that\nis, they were the targets the user put on the command line)."}),"\n"]}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.h3,{id:"second-order-implications",children:"Second-order implications"}),"\n",(0,i.jsx)(n.h4,{id:"non-determinism",children:"Non-determinism"}),"\n",(0,i.jsx)(n.p,{children:"Non-determinism in a build affects Buck2 and Buck1 differently. One scenario\nthat often works fine in Buck1 but can work catastrophically bad in Buck2 is a\ncodegen step, driven by a Python binary."}),"\n",(0,i.jsxs)(n.p,{children:["In certain configurations/modes, Python binaries are non-deterministic, because\nthey are XARs\n([",(0,i.jsx)(n.a,{href:"https://engineering.fb.com/2018/07/13/data-infrastructure/xars-a-more-efficient-open-source-system-for-self-contained-executables/%5D(eXecutable",children:"https://engineering.fb.com/2018/07/13/data-infrastructure/xars-a-more-efficient-open-source-system-for-self-contained-executables/](eXecutable"}),"\nARchives)) and that is always non-deterministic, which is bad!"]}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"In Buck1, that doesn\u2019t really matter, because you can get a cache hit on the\ncodegen output without ever visiting the XAR (as long as the input files\nhaven\u2019t changed)."}),"\n",(0,i.jsxs)(n.li,{children:["In Buck2, you need the XAR to check the action cache for the codegen step.","\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"However, binaries are often not cached in certain configurations/modes, so\nyour XAR isn\u2019t cached."}),"\n",(0,i.jsx)(n.li,{children:"Therefore, since your XAR build is non-deterministic, you\u2019ll always miss in\nthe action cache and the codegen step will always have to run in every\nbuild."}),"\n"]}),"\n"]}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"It can get worse! If the Python binary produces non-deterministic codegen, then\nthe entire build might become uncacheable."}),"\n",(0,i.jsx)(n.h4,{id:"cache-misses-dont-necessarily-propagate",children:"Cache misses don\u2019t necessarily propagate"}),"\n",(0,i.jsx)(n.p,{children:"Say that, in Buck2, you\u2019re trying to build a chain of actions like codegen ->\ncompile -> link."}),"\n",(0,i.jsx)(n.p,{children:"Even if your codegen step isn\u2019t cached (say, because its action inputs are\nnon-deterministic as mentioned above), as long as the codegen output is\ndeterministic, you can still get cache hits from compile and link steps."}),"\n",(0,i.jsx)(n.h4,{id:"hybrid-execution",children:"Hybrid execution"}),"\n",(0,i.jsx)(n.p,{children:"If you squint, you\u2019ll note that Buck1\u2019s build could be viewed as 'local first',\nwhereas Buck2\u2019s would be better viewed as 'remote first':"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"When Buck1 builds something remotely or gets a cache hit, the outputs are\nalways downloaded."}),"\n",(0,i.jsx)(n.li,{children:"When Buck2 builds something remotely or gets a cache hit, the outputs are\nnever downloaded."}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"In turn, this has some important implications:"}),"\n",(0,i.jsxs)(n.ul,{children:["\n",(0,i.jsx)(n.li,{children:"When Buck1 builds something locally, the inputs are always already present."}),"\n",(0,i.jsx)(n.li,{children:"When Buck2 builds something locally, the inputs have to be downloaded, unless\nthey were built locally (which if you\u2019re doing any RE, is usually not the\ncase), or if another command caused them to be downloaded."}),"\n"]}),"\n",(0,i.jsx)(n.p,{children:"This means that, in Buck1, running something locally when you have spare\nresources is usually a no-brainer, because it\u2019s always ready to go, and you\u2019ll\nsave on not having to download the output from RE (though you might have to\nupload the output if you need to run actions depending on it later)."}),"\n",(0,i.jsx)(n.p,{children:"On the flip side, with Buck2, that\u2019s not necessarily the case. To run an action\nlocally, you need to download inputs that you might otherwise not have needed,\nwhich will tax your network connection."})]})}function d(e={}){const{wrapper:n}={...(0,s.R)(),...e.components};return n?(0,i.jsx)(n,{...e,children:(0,i.jsx)(h,{...e})}):h(e)}},28453:(e,n,t)=>{t.d(n,{R:()=>c,x:()=>a});var i=t(96540);const s={},l=i.createContext(s);function c(e){const n=i.useContext(l);return i.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function a(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:c(e.components),i.createElement(l.Provider,{value:n},e.children)}}}]);