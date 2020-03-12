+++
title = "Chef Habitat and Provisioning Tools"
description = "Chef Habitat and Provisioning Tools"
+++

# <a name="habitat-and-provisioning" id="habitat-and-provisioning" data-magellan-target="habitat-and-provisioning" type="anchor">Chef Habitat and Provisioning Tools</a>

**Examples: [Terraform](https://www.terraform.io/) and [CloudFormation](https://aws.amazon.com/cloudformation/)**

Provisioning tools like Terraform or CloudFormation enable you to write a configuration file to manage infrastructure resources. The configuration file is used along with a CLI tool to create, read, update, and delete infrastructure resources in a declarative way. Chef Habitat is not a provisioning tool and works well with the provisioning tool of your choice.

Provisioning tools allow you to automate the installation and configuration of the Chef Habitat Supervisor, along with loading any applications and services you need to run. The [Terraform Chef Habitat Provisioner](https://www.terraform.io/docs/provisioners/habitat.html) provides a Terraform native method of installing the Chef Habitat Supervisor and managing Chef Habitat services. The [Chef Habitat Operator](https://www.habitat.sh/get-started/kubernetes/) provides a native method of auto-managing Chef Habitat services on Kubernetes. For any other provisioners, you can write your own script and include it in your automated provisioning. Visit the [Using Chef Habitat](https://www.habitat.sh/docs/using-habitat/) section of the docs to find more details about configuring the Chef Habitat Supervisor and Chef Habitat services.
