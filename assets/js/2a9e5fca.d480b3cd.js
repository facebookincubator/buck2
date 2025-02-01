"use strict";(self.webpackChunkwebsite=self.webpackChunkwebsite||[]).push([[7147],{55908:(e,n,t)=>{t.r(n),t.d(n,{assets:()=>r,contentTitle:()=>i,default:()=>m,frontMatter:()=>s,metadata:()=>l,toc:()=>c});var o=t(74848),a=t(28453);const s={id:"log",title:"log"},i=void 0,l={id:"users/commands/log",title:"log",description:"These are the flags/commands under buck2 log and their --help output:",source:"@site/../docs/users/commands/log.generated.md",sourceDirName:"users/commands",slug:"/users/commands/log",permalink:"/docs/users/commands/log",draft:!1,unlisted:!1,tags:[],version:"current",frontMatter:{id:"log",title:"log"},sidebar:"main",previous:{title:"killall",permalink:"/docs/users/commands/killall"},next:{title:"lsp",permalink:"/docs/users/commands/lsp"}},r={},c=[];function d(e){const n={code:"code",p:"p",pre:"pre",...(0,a.R)(),...e.components};return(0,o.jsxs)(o.Fragment,{children:[(0,o.jsxs)(n.p,{children:["These are the flags/commands under ",(0,o.jsx)(n.code,{children:"buck2 log"})," and their ",(0,o.jsx)(n.code,{children:"--help"})," output:"]}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Commands for interacting with buck2 logs\n\nUsage: buck2-release log [OPTIONS] <COMMAND>\n\nCommands:\n  what-ran           Output everything Buck2 ran from selected invocation\n  what-failed        Outputs every command that failed in the selected invocation\n  path               Output the path to the selected log\n  show               Outputs the log in JSON format from selected invocation\n  cmd                Show buck command line arguments from selected invocation\n  what-up            Show the spans that were open when the log ended\n  what-materialized  Outputs materializations from selected invocation\n  what-uploaded      Outputs stats about uploads to RE from the selected invocation\n  critical-path      Show the critical path for a selected build\n  replay             Replay an event log\n  show-user          Converts the event log from a selected invocation into a user event log, in\n                     JSONL format\n  summary            Outputs high level statistics about the build\n  diff               Subcommands for diff'ing two buck2 commands\n  help               Print this message or the help of the given subcommand(s)\n\nOptions:\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Output everything Buck2 ran from selected invocation.\n\nThe output is presented as a series of tab-delimited records with the following structure:\n\nThe reason for executing a given command. That's either to build or to test.\n\nThe identity of this command. This will include the target that ran required it.\n\nThe executor for this command. This will either be RE or local.\n\nDetails to reproduce it. For RE, that's the action digest. For local, the command.\n\nTo reproduce an action that ran on RE, use the following command then follow the instructions. The\nDIGEST is of the form `hash:size`.\n\nfrecli cas download-action DIGEST\n\nTo reproduce an action that ran locally, make sure your working directory is the project root (if\nunsure, use `buck2 root --kind project` to find it), then run the command. The command is already\nshell-quoted.\n\nUsage: buck2-release log what-ran [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n      --format <OUTPUT>\n          Which output format to use for this command\n          \n          [default: tabulated]\n          [possible values: tabulated, json, csv]\n\n      --emit-cache-queries\n          \n\n      --skip-cache-hits\n          \n\n      --skip-remote-executions\n          \n\n      --skip-local-executions\n          \n\n      --filter-category <FILTER_CATEGORY>\n          Regular expression to filter commands by given action category (i.e. type of of actions\n          that are similar but operate on different inputs, such as invocations of a C++ compiler\n          (whose category would be `cxx_compile`)). Matches by full string\n\n      --failed\n          Show only commands that failed\n\n      --incomplete\n          Show only commands that were not completed. That is command were running if buck2 process\n          was killed, or command currently running if buck2 is running build now\n\n      --show-std-err\n          Show also std_err from commands that are run. If the command fails before completing, we\n          display \"<command did not finish executing>\". If it finishes but there is no error, we\n          display \"<stderr is empty>\". Otherwise, std_err is shown. For JSON, we show raw values and\n          null for non-completion\n\n      --omit-empty-std-err\n          Omit commands if their std_err is empty\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Outputs every command that failed in the selected invocation.\n\nLook at the help for what-ran to understand the output format.\n\nUsage: buck2-release log what-failed [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n      --format <OUTPUT>\n          Which output format to use for this command\n          \n          [default: tabulated]\n          [possible values: tabulated, json, csv]\n\n      --emit-cache-queries\n          \n\n      --skip-cache-hits\n          \n\n      --skip-remote-executions\n          \n\n      --skip-local-executions\n          \n\n      --filter-category <FILTER_CATEGORY>\n          Regular expression to filter commands by given action category (i.e. type of of actions\n          that are similar but operate on different inputs, such as invocations of a C++ compiler\n          (whose category would be `cxx_compile`)). Matches by full string\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Output the path to the selected log\n\nUsage: buck2-release log path [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n      --all\n          List all the logs\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Outputs the log in JSON format from selected invocation\n\nUsage: buck2-release log show [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Show buck command line arguments from selected invocation.\n\nThis command output is not machine readable. Robots, please use `buck2 log show`.\n\nUsage: buck2-release log cmd [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n      --expand\n          Show @-expanded command line arguments instead of the original command line\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Show the spans that were open when the log ended\n\nUsage: buck2-release log what-up [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n      --after <NUMBER>\n          Print the actions that where open after certain amount of milliseconds\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Outputs materializations from selected invocation.\n\nThe output is a tab-separated list containing the path, the materialization method, the file count,\nand the total size (after decompression).\n\nUsage: buck2-release log what-materialized [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n  -s, --sort-by-size\n          Sort the output by total bytes in ascending order\n\n      --aggregate-by-ext\n          Aggregates the output by file extension\n\n      --format <OUTPUT>\n          Which output format to use for this command\n          \n          [default: tabulated]\n          [possible values: tabulated, json, csv]\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Outputs stats about uploads to RE from the selected invocation\n\nUsage: buck2-release log what-uploaded [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n      --format <OUTPUT>\n          Which output format to use for this command\n          \n          [default: tabulated]\n          [possible values: tabulated, json, csv]\n\n      --aggregate-by-ext\n          Aggregates the output by file extension\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Show the critical path for a selected build.\n\nThis produces tab-delimited output listing every node on the critical path.\n\nIt includes the kind of node, its name, category and identifier, as well as total duration (runtime\nof this node), user duration (duration the user can improve) and potential improvement before this\nnode stops being on the critical path.\n\nAll durations are in microseconds.\n\nUsage: buck2-release log critical-path [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n      --format <FORMAT>\n          Which output format to use for this command\n          \n          [default: tabulated]\n          [possible values: tabulated, json, csv]\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Replay an event log.\n\nThis command allows visualizing an existing event log in a Superconsole.\n\nUsage: buck2-release log replay [OPTIONS] [PATH] [OVERRIDE_ARGS]...\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\n  [OVERRIDE_ARGS]...\n          Override the arguments\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n      --speed <NUMBER>\n          Control the playback speed using a float (i.e. 0.5, 2, etc)\n\n      --preload\n          Preload the event log. This is typically only useful for benchmarking\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nConsole Options:\n      --console <super|simple|...>\n          Which console to use for this command\n          \n          [env: BUCK_CONSOLE=]\n          [default: auto]\n          [possible values: auto, none, simple, simplenotty, simpletty, super]\n\n      --ui <UI>...\n          Configure additional superconsole ui components.\n          \n          Accepts a comma-separated list of superconsole components to add. Possible values are:\n          \n          dice - shows information about evaluated dice nodes debugevents - shows information about\n          the flow of events from buckd\n          \n          These components can be turned on/off interactively. Press 'h' for help when superconsole\n          is active.\n\n          Possible values:\n          - dice\n          - debugevents\n          - io:          I/O panel\n          - re:          RE panel\n\n      --no-interactive-console\n          Disable console interactions\n          \n          [env: BUCK_NO_INTERACTIVE_CONSOLE=]\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Converts the event log from a selected invocation into a user event log, in JSONL format\n\nUsage: buck2-release log show-user [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Outputs high level statistics about the build\n\nUsage: buck2-release log summary [OPTIONS] [PATH]\n\nArguments:\n  [PATH]\n          A path to an event-log file to read from\n\nOptions:\n      --recent <NUMBER>\n          Open the event-log file from a recent command\n\n      --trace-id <ID>\n          Show log by trace id\n\n      --allow-remote\n          This option does nothing\n\n      --no-remote\n          Do not allow downloading the log from manifold if it's not found locally\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Subcommands for diff'ing two buck2 commands\n\nUsage: buck2-release log diff [OPTIONS] <COMMAND>\n\nCommands:\n  action-divergence  Identifies the first divergent action between two builds. Divergence is\n                     identified by the same action having differing outputs. Useful for identifying\n                     non-determinism\n  external-configs   Identifies the diff between external buckconfigs between two commands\n  help               Print this message or the help of the given subcommand(s)\n\nOptions:\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Identifies the first divergent action between two builds. Divergence is identified by the same\naction having differing outputs. Useful for identifying non-determinism\n\nUsage: buck2-release log diff action-divergence [OPTIONS] <--path1 <PATH1>|--trace-id1 <TRACE_ID1>|--recent1 <NUMBER>> <--path2 <PATH2>|--trace-id2 <TRACE_ID2>|--recent2 <NUMBER>>\n\nOptions:\n      --path1 <PATH1>\n          A path to an event-log file of the first command\n\n      --trace-id1 <TRACE_ID1>\n          Trace id of the first command\n\n      --recent1 <NUMBER>\n          Open the event-log file from a recent command for the first command\n\n      --path2 <PATH2>\n          A path to an event-log file of the second command\n\n      --trace-id2 <TRACE_ID2>\n          Trace id of the second command\n\n      --recent2 <NUMBER>\n          Open the event-log file from a recent command for the second command\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})}),"\n",(0,o.jsx)(n.pre,{children:(0,o.jsx)(n.code,{className:"language-text",children:"Identifies the diff between external buckconfigs between two commands\n\nUsage: buck2-release log diff external-configs [OPTIONS] <--path1 <PATH1>|--trace-id1 <TRACE_ID1>|--recent1 <NUMBER>> <--path2 <PATH2>|--trace-id2 <TRACE_ID2>|--recent2 <NUMBER>>\n\nOptions:\n      --path1 <PATH1>\n          A path to an event-log file of the first command\n\n      --trace-id1 <TRACE_ID1>\n          Trace id of the first command\n\n      --recent1 <NUMBER>\n          Open the event-log file from a recent command for the first command\n\n      --path2 <PATH2>\n          A path to an event-log file of the second command\n\n      --trace-id2 <TRACE_ID2>\n          Trace id of the second command\n\n      --recent2 <NUMBER>\n          Open the event-log file from a recent command for the second command\n\n  -h, --help\n          Print help (see a summary with '-h')\n\nUniversal Options:\n  -v, --verbose <VERBOSITY>\n          How verbose buck should be while logging.\n          \n          Values: 0 = Quiet, errors only; 1 = Show status. Default; 2 = more info about errors; 3 =\n          more info about everything; 4 = more info about everything + stderr;\n          \n          It can be combined with specific log items (stderr, full_failed_command, commands,\n          actions, status, stats, success) to fine-tune the verbosity of the log. Example usage\n          \"-v=1,stderr\"\n          \n          [default: 1]\n\n      --oncall <ONCALL>\n          The oncall executing this command\n\n      --client-metadata <CLIENT_METADATA>\n          Metadata key-value pairs to inject into Buck2's logging. Client metadata must be of the\n          form `key=value`, where `key` is a snake_case identifier, and will be sent to backend\n          datasets\n\n"})})]})}function m(e={}){const{wrapper:n}={...(0,a.R)(),...e.components};return n?(0,o.jsx)(n,{...e,children:(0,o.jsx)(d,{...e})}):d(e)}},28453:(e,n,t)=>{t.d(n,{R:()=>i,x:()=>l});var o=t(96540);const a={},s=o.createContext(a);function i(e){const n=o.useContext(s);return o.useMemo((function(){return"function"==typeof e?e(n):{...n,...e}}),[n,e])}function l(e){let n;return n=e.disableParentContext?"function"==typeof e.components?e.components(a):e.components||a:i(e.components),o.createElement(s.Provider,{value:n},e.children)}}}]);