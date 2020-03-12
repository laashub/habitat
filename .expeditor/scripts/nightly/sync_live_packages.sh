#!/bin/bash

set -euo pipefail

# For now*, we don't want to upload nightly packages to builder.  This means we need to sync 
# the stable channel from live builder to acceptance before we start building so that we are 
# using the same versions of our dependencies. This covers our desire to ensure packages remain
# in sync between acceptance and live. Ideally that could remain its own concern, however it 
# must be done before this pipeline starts, so making it the first step here seems correct.

# * This is being written in the context of the core-plans refresh so we need to build against
# packages that haven't been released yet. In order to prevent an accidental promotion of the
# packages, we choose to use acceptance. 

# Prerequisites for moving off of acceptance for nightly builds
# * core plans refresh is finished and promoted to stable
# * 

# TODO: This feels like behavior we want in `hab pkg download`, ex: `--sync-channel="stable"`

readonly pkg_cache="nightly-pipeline-sync-cache"

hab pkg download \
  --file .expeditor/files/nightly/package-sync-list.toml \
  --download-directory "$pkg_cache" \
  --channel "$BLDR_SOURCE_CHANNEL" \
  --url https://bldr.habitat.sh \
  --ignore-missing-seeds

# TODO: Does it make sense to remove the hard-coded live->acceptance sync
# in favor of using variables? For now, I don't think so as we only have two
# builder instances, and we always want to sync FROM live TO acceptance, 
# not the other way. 
hab pkg bulkupload \
  --auth "$PIPELINE_HAB_AUTH_TOKEN" \
  --channel stable \
  --url https://bldr.acceptance.habitat.sh \
  "$pkg_cache"
  
