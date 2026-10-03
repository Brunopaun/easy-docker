#![allow(dead_code)]

use bollard::Docker;
use bollard::models::{ContainerSummary, Volume, Network, ImageSummary};
use bollard::query_parameters::{
    ListContainersOptions, ListImagesOptions, ListNetworksOptionsBuilder, ListVolumesOptionsBuilder, RemoveContainerOptions, RemoveImageOptions, RestartContainerOptions, StartContainerOptions, StopContainerOptions
};

pub async fn get_docker_client() -> Result<Docker, bollard::errors::Error> {
    Docker::connect_with_local_defaults()
}

pub async fn list_containers(
    docker: &Docker,
) -> Result<Vec<ContainerSummary>, bollard::errors::Error> {
    let options = ListContainersOptions {
        all: true,
        ..Default::default()
    };

    docker.list_containers(Some(options)).await
}

pub async fn start_container(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    docker
        .start_container(id, None::<StartContainerOptions>)
        .await
}

pub async fn stop_container(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    docker
        .stop_container(id, None::<StopContainerOptions>)
        .await
}

pub async fn restart_container(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    docker
        .restart_container(id, None::<RestartContainerOptions>)
        .await
}

pub async fn remove_container(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    let _ = stop_container(docker, id).await;
    let options = RemoveContainerOptions {
        force: true,
        ..Default::default()
    };
    docker.remove_container(id, Some(options)).await
}

pub async fn list_volumes(docker: &Docker) -> Result<Vec<Volume>, bollard::errors::Error> {
    let options = ListVolumesOptionsBuilder::default().build();

    let response = docker.list_volumes(Some(options)).await?;
    Ok(response.volumes.unwrap_or_default())
}

pub async fn remove_volume(docker: &Docker, name: &str) -> Result<(), bollard::errors::Error> {
    let options = bollard::query_parameters::RemoveVolumeOptions {
        force: false,
    };
    docker.remove_volume(name, Some(options)).await
}

pub async fn list_images(docker: &Docker) -> Result<Vec<ImageSummary>, bollard::errors::Error> {
    let options = ListImagesOptions {
        all: true,
        ..Default::default()
    };

    docker.list_images(Some(options)).await
}

pub async fn remove_image(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    let options = RemoveImageOptions {
        force: false,
        ..Default::default()
    };
    let _ = docker.remove_image(id, Some(options), None).await?;
    Ok(())
}

pub async fn list_networks(docker: &Docker) -> Result<Vec<Network>, bollard::errors::Error> {
    let options = ListNetworksOptionsBuilder::default().build();

    docker.list_networks(Some(options)).await
}

pub async fn remove_network(docker: &Docker, id: &str) -> Result<(), bollard::errors::Error> {
    docker.remove_network(id).await
}

