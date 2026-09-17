import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { composeCompletion } from "./lib/completion.js";

test("bash completion preserves harness commands and shifts the product namespace", { skip: process.platform === "win32" }, () => {
  const native = '_airs() { COMPREPLY=(env login cli); }\ncomplete -F _airs airs\n';
  const product = String.raw`
_airs-cli_completions() {
  if [[ "\${COMP_WORDS[1]}" == runtime && "$COMP_CWORD" == 2 ]]; then
    COMPREPLY=(profiles topics)
  else
    COMPREPLY=(runtime tenant doctor)
  fi
}
complete -F _airs-cli_completions airs-cli
`.replaceAll("\\${", "${");
  const script = composeCompletion("bash", native, product) + String.raw`
COMP_WORDS=(airs ""); COMP_CWORD=1; _airs; printf '%s\n' "\${COMPREPLY[*]}"
COMP_WORDS=(airs cli runtime ""); COMP_CWORD=3; _airs; printf '%s\n' "\${COMPREPLY[*]}"
`.replaceAll("\\${", "${");
  const result = spawnSync("bash", ["-c", script], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.stdout.trim().split("\n"), ["env login cli", "profiles topics"]);
});

test("fish product suggestions require cli and do not register airs-cli", () => {
  const product = "function __airs-cli_using\nset -l tokens (commandline -opc)\nfor w in $tokens[2..-1]\nend\nend\ncomplete -c airs-cli -n '__airs-cli_using runtime' -a profiles\n";
  const result = composeCompletion("fish", "# native commands\n", product);
  assert.match(result, /test "\$tokens\[2\]" = cli; or return 1/);
  assert.match(result, /\$tokens\[3\.\.-1\]/);
  assert.doesNotMatch(result, /complete -c airs-cli/);
});

test("generated completions omit the legacy product root and harness-only CLI flags", () => {
  const bash = composeCompletion("bash", '_airs() { opts="env runtime cli"; }', "");
  assert.match(bash, /opts="env cli"/);
  const zsh = composeCompletion("zsh", "'cli:Product CLI' \\\n'runtime:' \\\n'env:Environments' \\\n", "");
  assert.doesNotMatch(zsh, /'runtime:'/);
  assert.match(zsh, /'env:Environments'/);
  const fish = composeCompletion("fish", 'complete -c airs -n "__fish_airs_needs_command" -a "runtime"\ncomplete -c airs -n "__fish_airs_using_subcommand cli" -l environment\ncomplete -c airs -n "__fish_airs_needs_command" -a "env"', "");
  assert.doesNotMatch(fish, /runtime|environment/);
  assert.match(fish, /-a "env"/);
});
