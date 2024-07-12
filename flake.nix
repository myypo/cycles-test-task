{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";

    disko.url = "github:nix-community/disko";
    disko.inputs.nixpkgs.follows = "nixpkgs";

    sops-nix.url = "github:Mic92/sops-nix";
    sops-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs:
    with inputs; let
      forEachSupportedSystem = let
        supportedSystems = ["x86_64-linux" "aarch64-linux"];
      in (f:
        nixpkgs.lib.genAttrs supportedSystems (system:
          f {
            pkgs = let
              overlays = [fenix.overlays.default];
            in
              import nixpkgs {inherit overlays system;};
          }));
    in {
      devShells = forEachSupportedSystem ({pkgs}:
        with pkgs; {
          default =
            mkShell.override {
              stdenv = stdenvAdapters.useMoldLinker clangStdenv;
            }
            mkShell {
              DATABASE_URL = "postgresql://postgres:postgres@127.0.0.1:5432/burger";

              packages = [
                openssl
                pkg-config

                rust-analyzer-nightly
                (fenix.complete.withComponents [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rust-std"
                  "rustc"
                  "rustfmt"
                  "llvm-tools-preview"
                  "rustc-codegen-cranelift-preview"
                ])

                just
                cargo-watch
                sqlx-cli
                hurl
                minio-client

                git-crypt
              ];
            };
        });

      packages = forEachSupportedSystem ({pkgs}:
        with pkgs; let
          build = args:
            rustPlatform.buildRustPackage (args
              // {
                name = "burger";
                cargoLock = {
                  lockFile = ./Cargo.lock;
                };
                src = lib.sources.sourceByRegex ./. ["Cargo.*" "(src|.sqlx|migrations|assets)(/.*)?"];
                doCheck = false;

                SQLX_OFFLINE = true;
                PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";

                postInstall = ''
                  mv assets $out
                '';
              });
        in rec {
          default = release;

          release = build {
            buildType = "release";
            buildFeatures = ["fixture"];

            nativeBuildInputs = [
              pkg-config

              (fenix.complete.withComponents [
                "rustc"
              ])
            ];
          };

          debug = build {
            buildType = "debug";
            buildFeatures = ["dev"];

            nativeBuildInputs = [
              pkg-config

              (fenix.complete.withComponents [
                "rustc"
                "llvm-tools-preview"
                "rustc-codegen-cranelift-preview"
              ])
            ];
          };

          e2eDocker = dockerTools.buildImage {
            name = "burger-e2e-server";
            tag = "latest";
            copyToRoot = [debug];
            config = {
              Cmd = ["${debug}/bin/burger"];
              Env = [
                "DATABASE_URL=postgresql://postgres:postgres@burger-e2e-database:6969/burger"
                "AWS_ENDPOINT_URL=http://burger-e2e-minio:9999/"
                "SERVER_ADDR=0.0.0.0:16161"

                "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
            };
          };

          releaseDocker = dockerTools.buildImage {
            name = "burger";
            tag = "latest";
            copyToRoot = with pkgs;
              buildEnv {
                name = "burger";
                pathsToLink = ["/bin"];
                paths = [
                  release
                ];
              };
            config = {
              Cmd = ["${release}/bin/burger"];
              WorkingDir = release;
              Env = [
                "SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
                "LD_LIBRARY_PATH=${lib.makeLibraryPath [openssl]}"
              ];
            };
          };
        });

      nixosConfigurations = {
        digital-ocean = let
          system = "x86_64-linux";
        in
          nixpkgs.lib.nixosSystem {
            inherit system;

            modules = [
              disko.nixosModules.disko
              {disko.devices.disk.disk1.device = "/dev/vda";}

              sops-nix.nixosModules.sops

              {
                networking.useDHCP = nixpkgs.lib.mkForce false;

                services.cloud-init = {
                  enable = true;
                  network.enable = true;
                };
              }

              (
                {
                  modulesPath,
                  lib,
                  pkgs,
                  config,
                  ...
                }: {
                  imports = [
                    (modulesPath + "/installer/scan/not-detected.nix")
                    (modulesPath + "/profiles/qemu-guest.nix")

                    (
                      {lib, ...}: {
                        disko.devices = {
                          disk.disk1 = {
                            device = lib.mkDefault "/dev/sda";
                            type = "disk";
                            content = {
                              type = "gpt";
                              partitions = {
                                boot = {
                                  name = "boot";
                                  size = "1M";
                                  type = "EF02";
                                };
                                esp = {
                                  name = "ESP";
                                  size = "500M";
                                  type = "EF00";
                                  content = {
                                    type = "filesystem";
                                    format = "vfat";
                                    mountpoint = "/boot";
                                  };
                                };
                                root = {
                                  name = "root";
                                  size = "100%";
                                  content = {
                                    type = "lvm_pv";
                                    vg = "pool";
                                  };
                                };
                              };
                            };
                          };
                          lvm_vg = {
                            pool = {
                              type = "lvm_vg";
                              lvs = {
                                root = {
                                  size = "100%FREE";
                                  content = {
                                    type = "filesystem";
                                    format = "ext4";
                                    mountpoint = "/";
                                    mountOptions = [
                                      "defaults"
                                    ];
                                  };
                                };
                              };
                            };
                          };
                        };
                      }
                    )
                  ];

                  boot.loader.grub = {
                    efiSupport = true;
                    efiInstallAsRemovable = true;
                  };
                  services.openssh.enable = true;
                  networking.firewall = {
                    enable = true;
                    allowedTCPPorts = [8080 9000];
                  };

                  virtualisation.oci-containers.containers = {
                    burger = {
                      image = "burger";
                      imageFile = self.packages.${system}.releaseDocker;
                      ports = ["8080:8080"];
                      environment = {
                        SERVER_ADDR = "0.0.0.0:8080";
                      };
                      environmentFiles = [
                        config.sops.templates."burger.env".path
                      ];
                      volumes = [
                        "/var/run/postgresql:/var/run/postgresql"
                      ];
                    };
                  };

                  services = {
                    postgresql = {
                      enable = true;
                      ensureDatabases = ["burger"];
                      ensureUsers = [
                        {
                          name = "burger";
                          ensureDBOwnership = true;
                        }
                      ];
                      authentication = lib.mkOverride 10 ''
                        #type database  DBuser  auth-method
                        local all       all     trust
                      '';
                    };
                    postgresqlBackup.enable = true;

                    minio = {
                      enable = true;
                      region = "minio";
                      rootCredentialsFile = config.sops.templates."minio.env".path;
                    };
                  };

                  environment.systemPackages = with pkgs; [
                    curl
                    gitMinimal
                    minio-client
                  ];

                  users.users.root.openssh.authorizedKeys.keys = [
                    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFHyTSwspx1XcWuVzDxOWhMBLJUT/s/PAdu1d3l4f65z nikirsmcgl@gmail.com"
                  ];

                  sops = {
                    defaultSopsFile = "${self}/secrets/.digital-ocean.yaml";

                    secrets = {
                      DATABASE_URL = {};
                      EDAMAM_ID_APP = {};
                      EDAMAM_KEY_APP = {};
                      AWS_ACCESS_KEY_ID = {};
                      AWS_SECRET_ACCESS_KEY = {};
                      AWS_ENDPOINT_URL = {};
                      AWS_REGION_NAME = {};
                      AWS_S3_BUCKET_NAME = {};
                    };

                    templates."burger.env" = {
                      content = with config.sops.placeholder; ''
                        DATABASE_URL=${DATABASE_URL}

                        EDAMAM_ID_APP=${EDAMAM_ID_APP}
                        EDAMAM_KEY_APP=${EDAMAM_KEY_APP}

                        AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}
                        AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}
                        AWS_ENDPOINT_URL=${AWS_ENDPOINT_URL}
                        AWS_REGION_NAME=${AWS_REGION_NAME}
                        AWS_S3_BUCKET_NAME=${AWS_S3_BUCKET_NAME}
                      '';
                    };
                    templates."minio.env" = {
                      content = with config.sops.placeholder; ''
                        MINIO_ROOT_USER=${AWS_ACCESS_KEY_ID}
                        MINIO_ROOT_PASSWORD=${AWS_SECRET_ACCESS_KEY}
                      '';
                    };
                  };

                  system.stateVersion = "24.11";
                }
              )
            ];
          };
      };
    };
}
