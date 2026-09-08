use super::*;
use crate::local::instance::Instance;

#[test]
fn isolated_ssh_ignores_user_config() {
    let flags = ssh_client_flags(std::path::Path::new("/tmp/missing-relay-key"), 1, 1);
    assert!(
        flags.windows(2).any(|pair| pair == ["-F", "/dev/null"]),
        "relay ssh must ignore ~/.ssh/config, flags={flags:?}"
    );
    assert!(
        flags.windows(2).any(|pair| pair == ["-o", "BatchMode=yes"]),
        "relay ssh must not prompt, flags={flags:?}"
    );
    assert!(
        flags
            .windows(2)
            .any(|pair| pair == ["-o", "IdentitiesOnly=yes"]),
        "relay ssh must use only the generated key, flags={flags:?}"
    );
}

#[test]
fn start_surfaces_ssh_stderr() {
    let instance = Instance::derive(
        Some(&format!("sdk-wh-{}", std::process::id())),
        Some(31_000),
    )
    .unwrap();
    let err = start(&instance).unwrap_err().to_string();
    let _ = std::fs::remove_dir_all(instance.artifact_dir());
    assert!(
        err.contains("Connection refused") || err.to_ascii_lowercase().contains("ssh:"),
        "expected the real ssh error, got {err}"
    );
}
