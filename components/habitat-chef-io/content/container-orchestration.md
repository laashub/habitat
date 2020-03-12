+++
title = "Container Orchestration with Chef Habitat"
description = "Container Orchestration with Chef Habitat"
+++

# <a name="container-orchestration" id="container-orchestration" data-magellan-target="container-orchestration">Container Orchestration with Chef Habitat</a>

Chef Habitat packages may be exported with the Supervisor directly into a [a variety of container formats](/docs/developing-packages/#pkg-exports), but frequently the container is running in a container orchestrator such as Kubernetes or Mesos. Container orchestrators provide scheduling and resource allocation, ensuring workloads are running and available. Containerized Chef Habitat packages can run within these runtimes, managing the applications while the runtimes handle the environment surrounding the application (ie. compute, networking, security).