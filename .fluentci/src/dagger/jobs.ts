import { dag } from "../../sdk/client.gen.ts";
import { buildRustFlags, getDirectory } from "./lib.ts";

export enum Job {
  test = "test",
  build = "build",
}

export const exclude = ["target", ".git", ".devbox", ".fluentci"];

export const test = async (src = ".", options: string[] = []) => {
  const context = await getDirectory(src);
  const ctr = dag
    .container()
    .from("rust:1.89-bullseye")
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withMountedCache("/app/target", dag.cacheVolume("target"))
    .withMountedCache("/root/cargo/registry", dag.cacheVolume("registry"))
    .withExec(["cargo", "test", ...options]);

  return ctr.stdout();
};

export const build = async (src = ".") => {
  const rustflags = buildRustFlags();
  const context = await getDirectory(src);
  const ctr = dag
    .container()
    .from("rust:1.89-bullseye")
    .withExec(["dpkg", "--add-architecture", "armhf"])
    .withExec(["dpkg", "--add-architecture", "arm64"])
    .withExec(["apt-get", "update"])
    .withExec(["apt-get", "install", "-y", "build-essential"])
    .withExec([
      "apt-get",
      "install",
      "-y",
      "-qq",
      "gcc-arm-linux-gnueabihf",
      "libc6-armhf-cross",
      "libc6-dev-armhf-cross",
      "gcc-aarch64-linux-gnu",
      "libc6-arm64-cross",
      "libc6-dev-arm64-cross",
      "libc6-armel-cross",
      "libc6-dev-armel-cross",
      "binutils-arm-linux-gnueabi",
      "gcc-arm-linux-gnueabi",
      "libncurses5-dev",
      "bison",
      "flex",
      "libssl-dev",
      "bc",
      "pkg-config",
      "libudev-dev",
    ])
    .withExec(["mkdir", "-p", "/build/sysroot"])
    .withDirectory("/app", context, { exclude })
    .withWorkdir("/app")
    .withMountedCache("/app/target", dag.cacheVolume("target"))
    .withMountedCache("/root/cargo/registry", dag.cacheVolume("registry"))
    .withMountedCache("/assets", dag.cacheVolume("gh-release-assets"))
    .withEnvVariable("RUSTFLAGS", rustflags)
    .withEnvVariable(
      "PKG_CONFIG_ALLOW_CROSS",
      Deno.env.get("TARGET") !== "x86_64-unknown-linux-gnu" ? "1" : "0"
    )
    .withEnvVariable(
      "C_INCLUDE_PATH",
      Deno.env.get("TARGET") !== "x86_64-unknown-linux-gnu"
        ? "/build/sysroot/usr/include"
        : "/usr/include"
    )
    .withEnvVariable("TAG", Deno.env.get("TAG") || "latest")
    .withEnvVariable(
      "TARGET",
      Deno.env.get("TARGET") || "x86_64-unknown-linux-gnu"
    )
    .withExec(["sh", "-c", "rustup target add $TARGET"])
    .withExec(["sh", "-c", "cargo build --release --target $TARGET"])
    .withExec(["sh", "-c", "cp target/${TARGET}/release/oh-my-droid ."])
    .withExec([
      "sh",
      "-c",
      "tar czvf /assets/oh-my-droid_${TAG}_${TARGET}.tar.gz oh-my-droid README.md LICENSE",
    ])
    .withExec([
      "sh",
      "-c",
      "shasum -a 256 /assets/oh-my-droid_${TAG}_${TARGET}.tar.gz > /assets/oh-my-droid_${TAG}_${TARGET}.tar.gz.sha256",
    ])
    .withExec(["sh", "-c", "cp /assets/oh-my-droid_${TAG}_${TARGET}.tar.gz ."])
    .withExec([
      "sh",
      "-c",
      "cp /assets/oh-my-droid_${TAG}_${TARGET}.tar.gz.sha256 .",
    ]);

  const exe = await ctr.file(
    `/app/oh-my-droid_${Deno.env.get("TAG")}_${Deno.env.get("TARGET")}.tar.gz`
  );
  await exe.export(
    `./oh-my-droid_${Deno.env.get("TAG")}_${Deno.env.get("TARGET")}.tar.gz`
  );

  const sha = await ctr.file(
    `/app/oh-my-droid_${Deno.env.get("TAG")}_${Deno.env.get(
      "TARGET"
    )}.tar.gz.sha256`
  );
  await sha.export(
    `./oh-my-droid_${Deno.env.get("TAG")}_${Deno.env.get(
      "TARGET"
    )}.tar.gz.sha256`
  );
  return ctr.stdout();
};

export type JobExec = (src?: string) =>
  | Promise<string>
  | ((
      src?: string,
      options?: {
        ignore: string[];
      }
    ) => Promise<string>);

export const runnableJobs: Record<Job, JobExec> = {
  [Job.test]: test,
  [Job.build]: build,
};

export const jobDescriptions: Record<Job, string> = {
  [Job.test]: "Run tests",
  [Job.build]: "Build the project",
};
