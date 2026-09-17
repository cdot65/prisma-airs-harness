import { spawnSync } from "node:child_process";

// Keep the native command tree authoritative and delegate only the exact `cli`
// namespace. Product completions must never replace harness root completions.
export function composeCompletion(shell, native, product) {
  if (shell === "bash") {
    const harness = native.replace(/\b_airs\b/g, "_airs_harness")
      .replace(/opts="([^"\n]*)"/g, (_, options) =>
        `opts="${options.split(" ").filter((item) => item !== "runtime").join(" ")}"`);
    const cli = product.replaceAll("_airs-cli_completions", "_airs_product")
      .replace(/^complete -F .*$/gm, "");
    return `${harness}\n${cli}\n_airs() {
  if [[ "\${COMP_WORDS[1]}" == cli ]]; then
    local -a COMP_WORDS=(airs-cli "\${COMP_WORDS[@]:2}")
    local COMP_CWORD=$((COMP_CWORD - 1))
    _airs_product "$@"
  else
    _airs_harness "$@"
  fi
}
complete -F _airs -o bashdefault -o default airs
`;
  }
  if (shell === "zsh") {
    const harness = native.replace(/\b_airs\b/g, "_airs_harness")
      .replace(/^'runtime:' \\\n/gm, "");
    const cli = product.replaceAll("_airs-cli", "_airs_product")
      .replace(/^#compdef .*$/gm, "").replace(/^_airs_product "\$@"$/gm, "");
    return `${harness}\n${cli}\n_airs() {
  if [[ "\${words[2]}" == cli ]]; then
    local -a words=(airs-cli "\${words[@]:2}")
    local CURRENT=$((CURRENT - 1))
    _airs_product "$@"
  else
    _airs_harness "$@"
  fi
}
compdef _airs airs
`;
  }
  if (shell === "fish") {
    const harness = native.split("\n").filter((line) =>
      !line.includes('__fish_airs_using_subcommand cli"') &&
      !line.includes('-a "runtime"')).join("\n");
    const cli = product.replaceAll("__airs-cli_using", "__airs_product_using")
      .replaceAll("complete -c airs-cli", "complete -c airs")
      .replace("set -l tokens (commandline -opc)",
        "set -l tokens (commandline -opc)\n    test \"$tokens[2]\" = cli; or return 1")
      .replace("$tokens[2..-1]", "$tokens[3..-1]");
    return `${harness}\n${cli}`;
  }
  return native;
}

export function runCompletion(binary, cli, shell, env) {
  const native = spawnSync(binary, ["completion", shell], {
    env, encoding: "utf8", timeout: 30000, maxBuffer: 8 * 1024 * 1024,
  });
  if (native.error || native.status !== 0) {
    throw new Error("Cannot generate harness completions.");
  }
  if (!["bash", "zsh", "fish"].includes(shell)) {
    process.stdout.write(native.stdout);
    return;
  }
  const product = spawnSync(process.execPath, [cli, "completion", shell], {
    env, encoding: "utf8", timeout: 30000, maxBuffer: 8 * 1024 * 1024,
  });
  if (product.error || product.status !== 0) {
    throw new Error("Cannot generate bundled CLI completions.");
  }
  process.stdout.write(composeCompletion(shell, native.stdout, product.stdout));
}
