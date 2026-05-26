{
  config,
  inputs,
  self,
  ...
}:
{
  perSystem =
    {
      config,
      system,
      pkgs,
      ...
    }:
    {
      packages = {
        default = config.packages.selector4nix;
        selector4nix = pkgs.callPackage ../package.nix { };
        selector4nix-static = pkgs.pkgsStatic.callPackage ../package.nix { };
      };

      legacyPackages = config.packages;
    };
}
