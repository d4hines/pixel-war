{
  pkgs,
  tezos-packages,
}:
pkgs.writeShellApplication {
  runtimeInputs = with pkgs; [
    vim # yes this is dumb I know but it pulls in xxd
    tezos-packages.trunk-octez-client
  ];
  name = "process-message";
  text = ''
    export PASSWORD=${../secret/password}
    ${builtins.readFile ../scripts/process.sh}
  '';
}
