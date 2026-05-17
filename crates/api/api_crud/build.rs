fn main() {
  let commit = std::env::var("BREHON_FORK_COMMIT")
    .or_else(|_| {
      std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .ok_or(std::env::VarError::NotPresent)
    })
    .unwrap_or_else(|_| "unknown".to_string());
  println!("cargo:rustc-env=BREHON_FORK_COMMIT={commit}");
  println!("cargo:rerun-if-env-changed=BREHON_FORK_COMMIT");
  println!("cargo:rerun-if-changed=../../../.git/HEAD");
}
