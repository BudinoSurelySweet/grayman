{
  description = "Everything you need to work with Rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = inputs: {
    # `nix shell` or `nix run` packages
    packages =
      builtins.mapAttrs (system: pkgs: {
        cargo = pkgs.cargo;
        rustc = pkgs.rustc;
        clippy = pkgs.clippy;

        default = inputs.self.packages.${system}.cargo;
      })
      inputs.nixpkgs.legacyPackages;

    # `nix develop` groups of packages
    devShells =
      builtins.mapAttrs (system: pkgs: {
        # Default group
        default = pkgs.mkShell {
          # Packages
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];

          # Welcome message
          shellHook = ''
            printf "\n🦀 Rust environment is ready.\n\n"
            cargo --version
          '';
        };
      })
      inputs.nixpkgs.legacyPackages;
  };
}
