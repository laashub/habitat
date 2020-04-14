#!/bin/bash

set -eou pipefail

tempdir="$(mktemp --directory --tmpdir="$(pwd)" -t "downloads-XXXX")"

all_targets=(x86_64-linux
             x86_64-linux-kernel2
             x86_64-windows)
linux_targets=(x86_64-linux
             x86_64-linux-kernel2)
not_kernel2_targets=(x86_64-linux
                     x86_64-windows)
hab_idents=(core/hab
            core/hab-studio
            core/hab-sup
            core/hab-launcher)
linux_idents=(core/hab-plan-build
              core/hab-backline)
win_idents=(core/hab-plan-build-ps1
            core/windows-service)
not_kernel2_idents=(core/hab-pkg-export-docker
                    core/hab-pkg-export-tar)

export HAB_AUTH_TOKEN="${PIPELINE_HAB_AUTH_TOKEN}"

for ident in "${hab_idents[@]}"
do
    for target in "${all_targets[@]}"
    do
        hab pkg download -t "${target}" --download-directory "${tempdir}" "${ident}"
    done
done

for ident in "${linux_idents[@]}"
do
    for target in "${linux_targets[@]}"
    do
        hab pkg download -t "${target}" --download-directory "${tempdir}" "${ident}"
    done
done

for ident in "${win_idents[@]}"
do
    hab pkg download -t x86_64-windows --download-directory "${tempdir}" "${ident}"
done

for ident in "${not_kernel2_idents[@]}"
do
    for target in "${not_kernel2_targets[@]}"
    do
        hab pkg download -t "${target}" --download-directory "${tempdir}" "${ident}"
    done
done

export HAB_AUTH_TOKEN="${PIPELINE_ACCEPTANCE_AUTH_TOKEN}"

for f in ${tempdir}/artifacts/*
do
    hab pkg upload -u https://bldr.acceptance.habitat.sh --cache-key-path "${tempdir}/keys" -c stable $f
done
