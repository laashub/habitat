# Utility function to create a custom tag for a container image that
# is unique in this test run.
function New-CustomTag {
    $timestamp = Get-Date -UFormat "%Y%m%d%H%M%S"
    "e2e-test-$timestamp"
}

# Export a core/redis container image.
#
# $extra_args are options for `hab pkg export docker` to influence
# container creation. (e.g., pass "--multi-layer" to create a
#multi-layer image.)
function New-Image() {
    param(
        [Parameter(Mandatory=$true)][string]$tag,
        [Parameter(Mandatory=$false)][string]$extra_args
    )
    Write-Host (hab pkg export docker core/nginx --tag-custom=$tag --base-pkgs-channel=$env:HAB_INTERNAL_BLDR_CHANNEL $extra_args | Out-String)
    "core/nginx:$tag"
}

# Run a given Habitat container image in the background, returning the
# name of the container.
#
# $extra_args are for `docker run` and can affect how the container is
# actually executed. (e.g., pass "--user=12354151" to see if the
# Supervisor can run as a non-root user)
function Start-Container() {
    param(
        [Parameter(Mandatory=$true)][string]$image,
        [Parameter(Mandatory=$false)][string]$extra_args
    )
    $name="e2e-docker-export-container"
    # We're using a non-standard port because we will also execute these
    # tests as a non-root user, and non-root users don't get to listen
    # on port 80.
    Write-Host (docker run -d -p 9999:9999 --name=$name --rm --env=HAB_LICENSE=accept-no-persist --env=HAB_NGINX='http.listen.port = 9999' $extra_args $image | Out-String)
    "$name"
}

# If we can set and get a value from Redis running in the container,
# then we know we created a container that can actually run.
function Confirm-ContainerBehavior() {
    param(
        [Parameter(Mandatory=$true)][string]$container
    )
    # Give 10 seconds for the container to come up and for Nginx to start
    Start-Sleep -Seconds 10

    # This will error with a 403 because nginx is not running any sites
    try  { Invoke-WebRequest "http://localhost:9999" }
    catch [Microsoft.PowerShell.Commands.HttpResponseException] { $headers = $_.Exception.Response.Headers }
    [string]$headers.Server | Should -BeLike "nginx/*"
}

Describe "hab pkg export docker" {
    BeforeAll {
        $tag = New-CustomTag
        $script:image = New-Image $tag
    }

    AfterAll {
        docker rmi $script:image
    }

    It "Creates an image" {
        docker inspect $script:image | Should -Not -Be $null
    }

    Describe "executing the container as root" {
        BeforeEach {
            $script:container = Start-Container $image
        }

        AfterEach {
            docker kill $script:container
        }

        It "works!" {
            Confirm-ContainerBehavior $script:container
        }
    }
    if($IsLinux) {
        Describe "executing a container as non-root" {
            BeforeEach {
                $script:container = Start-Container $script:image "--user=8888888"
            }

            AfterEach {
                docker kill $script:container
            }

            It "works!" {
                Confirm-ContainerBehavior $script:container
            }
        }
    }
}

Describe "hab pkg export docker --multi-layer" {
    BeforeAll {
        $tag = New-CustomTag
        $script:image = New-Image $tag "--multi-layer"
    }

    AfterAll {
        docker rmi $script:image
    }

    It "Creates an image" {
        docker inspect $script:image | Should -Not -Be $null
    }

    Describe "executing the container as root" {
        BeforeEach {
            $script:container = Start-Container $script:image
        }

        AfterEach {
            docker kill $script:container
        }

        It "works!" {
            Confirm-ContainerBehavior $script:container
        }
    }

    if($IsLinux) {
        Describe "executing a container as non-root" {
            BeforeEach {
                $script:container = Start-Container $script:image "--user=8888888"
            }

            AfterEach {
                docker kill $script:container
            }

            It "works!" {
                Confirm-ContainerBehavior $script:container
            }
        }
    }
}
