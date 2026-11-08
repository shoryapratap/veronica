/// Analyzes the host machine to detect the OS, architecture, and whether Robot OS is active.
pub fn detect_environment() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // Check if Robot OS (ROS 2) environment variable is present
    let ros_distro = std::env::var("ROS_DISTRO").ok();

    match (os, ros_distro) {
        ("linux", Some(distro)) => {
            format!("Linux ({}) with Robot OS (ROS 2 {}) active", arch, distro)
        }
        (other_os, _) => {
            format!("{} ({})", other_os, arch)
        }
    }
}
