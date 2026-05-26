{
  config,
  inputs,
  self,
  ...
}:
{
  options.inputs = inputs.nixpkgs.lib.mkOption {
    type = inputs.nixpkgs.lib.types.raw;
    readOnly = true;
  };

  config.inputs = inputs;
}
