use std::{println, process::Command};

fn main() {
    let output = Command::new("docker")
        .args(["ps", "-a", "-q"])
        .output()
        .expect("Fail to execute docker ps");

    let ids_str = String::from_utf8_lossy(&output.stdout);
    let containers: Vec<&str> = ids_str.trim().split_whitespace().collect();

    if !containers.is_empty() {
        let mut docker = Command::new("docker");
        docker.arg("rm").arg("-f").args(&containers);

        match docker.status() {
            Ok(status) if status.success() => println!("All docker containers cleaned"),
            Ok(status) => println!("Failed to clean containers: exit status {}", status),
            Err(err) => println!("Failed to execute docker command: {}", err),
        }
    }   
    
    let mut prune = Command::new("docker");
    prune.arg("network").arg("prune").arg("-f");

    match prune.status() {
        Ok(status) if status.success() => println!("All networks pruned"),
        Ok(status) => println!("Failed to prune networks: exit status {}", status),
        Err(err) => println!("Failed to execute docker command: {}", err),
    }
    let volumes = Command::new("docker")
    .args(["volume", "ls", "-q"])
    .output()
    .expect("Fail to execute docker ps");

    let volumes_ids_str = String::from_utf8_lossy(&volumes.stdout);

    let volumes_ids: Vec<&str> = volumes_ids_str.trim().split_whitespace().collect();

    if !volumes_ids.is_empty() {
        let mut docker = Command::new("docker");
        docker.arg("volume").arg("rm").args(&volumes_ids);

        match docker.status() {
            Ok(status) if status.success() => println!("All docker volumes cleaned"),
            Ok(status) => println!("Failed to clean volume: exit status {}", status),
            Err(err) => println!("Failed to execute docker command: {}", err),
        }
    } 
}
