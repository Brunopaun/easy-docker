#![allow(dead_code)]

use bollard::Docker;
use bollard::models::ContainerSummary;
use bollard::query_parameters::{
    ListContainersOptions, StartContainerOptions, StopContainerOptions,
    RestartContainerOptions, RemoveContainerOptions,
};

pub async fn get_docker_client() -> Result<Docker, bollard::errors::Error> {
    Docker::connect_with_local_defaults()
}

pub async fn list_containers(docker: &Docker) -> Result<Vec<ContainerSummary>, bollard::errors::Error> {
    let options = ListContainersOptions {
        all: true,
        ..Default::default()
    };

    docker.list_containers(Some(options)).await
}

pub async fn start_container(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    docker.start_container(id, None::<StartContainerOptions>).await
}

pub async fn stop_container(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    docker.stop_container(id, None::<StopContainerOptions>).await
}

pub async fn restart_container(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    docker.restart_container(id, None::<RestartContainerOptions>).await
}

pub async fn remove_container(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    let _ = stop_container(docker, id).await;
    let options = RemoveContainerOptions {
        force: true,
        ..Default::default()
    };
    docker.remove_container(id, Some(options)).await
}