+++
title = "Developing Packages"
description = "Documentation for writing Chef Habitat Plan files including configuration templates, binds, and exporting"
+++
# <a name="developing-packages" id="developing-packages" data-magellan-target="developing-packages">Develop Packages</a>

In Chef Habitat the unit of automation is the application itself. This chapter includes content related specifically to the process and workflow of developing a plan that will instruct Chef Habitat in how to build, deploy, and manage your application.

## <a name="write-plans" id="write-plans" data-magellan-target="write-plans" type="anchor">Writing Plans</a>

Artifacts are the cryptographically-signed tarballs that are uploaded, downloaded, unpacked, and installed in Chef Habitat. They are built from shell scripts known as plans, but may also include application lifecycle hooks and service configuration files that describe the behavior and configuration of a running service.

At the center of Chef Habitat packaging is the plan. This is a directory comprised of shell scripts and optional configuration files that define how you download, configure, make, install, and manage the lifecycle of the software in the artifact. More conceptual information on artifacts can be found in the [Artifacts glossary topic](/docs/glossary#glossary-artifacts).

As a way to start to understand plans, let's look at an example `plan.sh` for [sqlite](http://www.sqlite.org/):

```bash plan.sh
pkg_name=sqlite
pkg_version=3130000
pkg_origin=core
pkg_license=('Public Domain')
pkg_maintainer="The Chef Habitat Maintainers <humans@habitat.sh>"
pkg_description="A software library that implements a self-contained, serverless, zero-configuration, transactional SQL database engine."
pkg_upstream_url=https://www.sqlite.org/
pkg_source=https://www.sqlite.org/2016/${pkg_name}-autoconf-${pkg_version}.tar.gz
pkg_filename=${pkg_name}-autoconf-${pkg_version}.tar.gz
pkg_dirname=${pkg_name}-autoconf-${pkg_version}
pkg_shasum=e2797026b3310c9d08bd472f6d430058c6dd139ff9d4e30289884ccd9744086b
pkg_deps=(core/glibc core/readline)
pkg_build_deps=(core/gcc core/make core/coreutils)
pkg_lib_dirs=(lib)
pkg_include_dirs=(include)
pkg_bin_dirs=(bin)
```

> Note: On Windows we would create a plan.ps1 file instead. All the variable names are the same but we use Powershell syntax so, for example, `pkg_deps=(core/glibc core/readline)` becomes `$pkg_deps=@("core/glibc", "core/readline")`.

It has the name of the software, the version, where to download it, a checksum to verify the contents are what we expect, run dependencies on `core/glibc` and `core/readline`, build dependencies on `core/coreutils`, `core/make`, `core/gcc`, libraries files in `lib`, header files in `include`, and a binary file in `bin`. Also, because it's a core plan, it has a description and upstream URL for the source project included.

> Note: The `core` prefix is the origin of those dependencies. For more information, see [Create an Origin](/docs/using-builder/#builder-origin)

When you have finished creating your plan and call `build` in Chef Habitat studio, the following occurs:

1. The build script ensures that the private origin key is available to sign the artifact.
2. If specified in `pkg_source`, a compressed file containing the source code is downloaded.
3. The checksum of that file, specified in `pkg_shasum`, is validated.
4. The source is extracted into a temporary cache.
5. Unless overridden, the callback methods will build and install the binary or library via `make` and `make install`, respectively for Linux based builds.
6. Your package contents (binaries, runtime dependencies, libraries, assets, etc.) are then compressed into a tarball.
7. The tarball is signed with your private origin key and given a `.hart` file extension.

After the build script completes, you can then upload your package to Chef Habitat Builder, or install and start your package locally.

> Note: The plan.sh or plan.ps1 file is the only required file to create a package. Configuration files, runtime hooks, and other source files are optional.

## <a name="plan-intro" id="plan-intro" data-magellan-target="plan-intro" type="anchor">Write Your First Plan</a>

All plans must have a `plan.sh` or `plan.ps1` at the root of the plan context. They may even include both if a package is targeting both Windows and Linux platforms. This file will be used by the `hab-plan-build` command to build your package. To create a plan, do the following:

1. If you haven't done so already, [download the `hab` CLI](/docs/install-habitat/) and install it per the instructions on the download page.

2. Run `hab cli setup` and follow the instructions in the setup script.

3. The easiest way to create a plan is to use the `hab plan init` subcommand. This subcommand will create a directory, known as the plan context, that contains your plan file and any runtime hooks and/or templated configuration data.

    To use `hab plan init` as part of your project repo, navigate to the root of your project repo and run `hab plan init`. It will create a new `habitat` sub-directory with a plan.sh (or plan.ps1 on Windows) based on the name of the parent directory, and include a `default.toml` file as well as `config` and `hooks` directories for you to populate as needed. For example:

    ```bash
    $ cd /path/to/<reponame>
    $ hab plan init
    ```

    will result in a new `habitat` directory located at `/path/to/<reponame>/habitat`. A plan file will be created and the `pkg_name` variable will be set to _\<reponame\>_. Also, any environment variables that you have previously set (such as `HAB_ORIGIN`) will be used to populate the respective `pkg_*` variables.

    If you want to auto-populate more of the `pkg_*` variables, you also have the option of setting them when calling `hab plan init`, as shown in the following example:

    ```bash
    $ env pkg_svc_user=someuser pkg_deps="(core/make core/coreutils)" \
       pkg_license="('MIT' 'Apache-2.0')" pkg_bin_dirs="(bin sbin)" \
       pkg_version=1.0.0 pkg_description="foo" pkg_maintainer="you" \
       hab plan init yourplan
    ```

     See [hab plan init](/docs/habitat-cli#hab-plan-init) for more information on how to use this subcommand.

4. Now that you have stubbed out your plan file in your plan context, open it and begin modifying it to suit your needs.

When writing a plan, it's important to understand that you are defining both how the package is built and the actions Chef Habitat will take when the Supervisor starts and manages the child processes in the package. The following sections explain what you need to do for each phase.

### Buildtime Workflow

For buildtime installation and configuration, workflow steps need to be included in the plan file to define how you will install your application source files into a package. Before writing your plan, you should know and understand how your application binaries are currently built, installed, what their dependencies are, and where your application or software library expects to find those dependencies.

The main steps in the buildtime workflow are the following:

1. Create your fully-qualified package identifier.
2. Add licensing and contact information.
3. Download and unpack your source files.
4. Define your dependencies.
5. (Optional) Override any default build phases you need to using callbacks.

The following sections describe each of these steps in more detail.

#### Create your Package Identifier

The origin is a place for you to set default privacy rules, store your packages, and collaborate with teammates. For example, the "core" origin is where the core maintainers of Chef Habitat share packages that are foundational to building other packages. If you would like to browse them, they are located in the [core-plans repo](https://github.com/habitat-sh/core-plans), and on [Chef Habitat Builder's Core Origin](https://bldr.habitat.sh/#/pkgs/core).

Creating artifacts for a specific origin requires that you have access to the that origin's private key. The private origin key will be used to sign the artifact when it is built by the `hab plan build` command. Origin keys are kept in `$HOME/.hab/cache/keys` on the host machine when running `hab` as a non-root user and `/hab/cache/keys` when running as root (including in the studio). For more information on origin keys, see [Keys](/docs/glossary/glossary-keys).

The next important part of your package identifier is the name of the package. Standard naming convention is to base the name of the package off of the name of the source or project you download and install into the package.

#### Add Licensing and Contact Information

You should enter your contact information in your plan.

Most importantly, you should update the `pkg_license` value to indicate the type of license (or licenses) that your source files are licensed under. Valid license types can be found at [https://spdx.org/licenses/](https://spdx.org/licenses/). You can include multiple licenses as an array.

> Note: Because all arrays in the pkg_* settings are shell arrays, they are whitespace delimited.

#### Download and Unpack Your Source Files

Add in the `pkg_source` value that points to where your source files are located at. Any `wget` url will work; however, unless you're downloading a tarball from a public endpoint, you may need to modify how you download your source files and where in your plan.sh you perform the download operation.

Chef Habitat supports retrieving source files from [GitHub](https://github.com). When cloning from GitHub, it is recommended to use https URIs because they are proxy friendly, whereas `git@github` or `git://` are not. To download the source from a GitHub repository, implement `do_download()` in your plan.sh (or `Invoke-Download` in a plan.ps1) and add a reference the `core/git` package as a build dependency. Because Chef Habitat does not contain a system-wide CA cert bundle, you must use the `core/cacerts` package and export the `GIT_SSL_CAINFO` environment variable to point the `core/cacerts` package on Linux. Here's an example of how to do this in the `do_download()` callback.

```bash
do_download() {
  export GIT_SSL_CAINFO="$(pkg_path_for core/cacerts)/ssl/certs/cacert.pem"
  git clone https://github.com/chef/chef
  pushd chef
  git checkout $pkg_version
  popd
  tar -cjvf $HAB_CACHE_SRC_PATH/${pkg_name}-${pkg_version}.tar.bz2 \
      --transform "s,^\./chef,chef-${pkg_version}," ./chef \
      --exclude chef/.git --exclude chef/spec
  pkg_shasum=$(trim $(sha256sum $HAB_CACHE_SRC_PATH/${pkg_filename} | cut -d " " -f 1))
}
```

The plan.ps1 equivalent would be:

```PS
Function Invoke-Download {
  git clone https://github.com/chef/chef
  pushd chef
  git checkout $pkg_version
  popd
  Compress-Archive -Path chef/* -DestinationPath $HAB_CACHE_SRC_PATH/$pkg_name-$pkg_version.zip -Force
  $script:pkg_shasum = (Get-FileHash -path $HAB_CACHE_SRC_PATH/$pkg_name-$pkg_version.zip -Algorithm SHA256).Hash.ToLower()
}
```

After you have either specified your source in `pkg_source`, or overridden the **do_download()** or **Invoke-Download** callback, create a sha256 checksum for your source archive and enter it as the `pkg_shasum` value. The build script will verify this after it has downloaded the archive.

> Note: If your computed value does not match the value calculated by the `hab-plan-build` script, an error with the expected value will be returned when you execute your plan.

If your package does not download any application or service source files, then you will need to override the **do_download()**, **do_verify()**, and **do_unpack()** callbacks. See [Callbacks](/docs/reference/build-phase-callbacks) for more details.

#### Define Your Dependencies

Applications have two types of dependencies: buildtime and runtime.

Declare any build dependencies in `pkg_build_deps` and any run dependencies in `pkg_deps`. You can include version and release information when declaring dependencies if your application is bound to a particular version.

The package `core/glibc` is typically listed as a run dependency and `core/coreutils` as a build dependency, however, you should not take any inference from this. There are no standard dependencies that every package must have. For example, the mytutorialapp package only includes the `core/node` as a run dependency. You should include dependencies that would natively be part of the build or runtime dependencies your application or service would normally depend on.

There is a third type of dependencies, transitive dependencies, that are the run dependencies of either the build or run dependencies listed in your plan. You do not need to explicitly declare transitive dependencies, but they are included in the list of files when your package is built. See [Package Contents](/docs/reference/package-contents) for more information.

#### Override Build Phase Defaults with Callbacks

As shown in an example above, there are occasions when you want to override the default behavior of the hab-plan-build script. The Plan syntax guide lists the default implementations for [build phase callbacks](/docs/reference/build-phase-callbacks), but if you need to reference specific packages in the process of building your applications or services, then you need to override the default implementations as in the example below.

```bash
pkg_name=httpd
pkg_origin=core
pkg_version=2.4.18
pkg_maintainer="The Chef Habitat Maintainers <humans@habitat.sh>"
pkg_license=('apache')
pkg_source=http://www.apache.org/dist/${pkg_name}/${pkg_name}-${pkg_version}.tar.gz
pkg_shasum=1c39b55108223ba197cae2d0bb81c180e4db19e23d177fba5910785de1ac5527
pkg_deps=(core/glibc core/expat core/libiconv core/apr core/apr-util core/pcre core/zlib core/openssl)
pkg_build_deps=(core/patch core/make core/gcc)
pkg_bin_dirs=(bin)
pkg_lib_dirs=(lib)
pkg_exports=(
  [port]=serverport
)
pkg_svc_run="httpd -DFOREGROUND -f $pkg_svc_config_path/httpd.conf"
pkg_svc_user="root"

do_build() {
  ./configure --prefix=$pkg_prefix \
              --with-expat=$(pkg_path_for expat) \
              --with-iconv=$(pkg_path_for libiconv) \
              --with-pcre=$(pkg_path_for pcre) \
              --with-apr=$(pkg_path_for apr) \
              --with-apr-util=$(pkg_path_for apr-util) \
              --with-z=$(pkg_path_for zlib) \
              --enable-ssl --with-ssl=$(pkg_path_for openssl) \
              --enable-modules=most --enable-mods-shared=most
  make
}
```

In this example, the `core/httpd` plan references several other core packages through the use of the `pkg_path_for` function before `make` is called. You can use a similar pattern if you need reference a binary or library when building your source files.

Or consider this override from a plan.ps1:

```PS
function Invoke-Build {
    Push-Location "$PLAN_CONTEXT"
    try {
        cargo build --release --verbose
        if($LASTEXITCODE -ne 0) {
            Write-Error "Cargo build failed!"
        }
    }
    finally { Pop-Location }
}
```

Here the plan is building an application written in Rust. So it overrides `Invoke-Build` and uses the `cargo` utility included in its buildtime dependency on `core/rust`.

> Note: Powershell plan function names differ from their Bash counterparts in that they use the `Invoke` `verb` instead of the `do_` prefix.

When overriding any callbacks, you may use any of the variables, settings, or functions in the [Plan syntax guide](/docs/reference/), except for the runtime template data. Those can only be used in Application Lifecycle hooks once a Chef Habitat service is running.

### Runtime Workflow

Similar to defining the setup and installation experience at buildtime, behavior for your application or service needs to be defined for the Supervisor. This is done at runtime through Application lifecycle hooks. See [Application Lifecycle hooks](/docs/reference/application-lifecycle-hooks) for more information and examples.

If you only need to start the application or service when the Chef Habitat service starts, you can instead use the `pkg_svc_run` setting and specify the command as a string. When your package is created, a basic run hook will be created by Chef Habitat.

You can use any of the [runtime configuration settings](/docs/reference/template-data), either defined by you in your config file, or defined by Chef Habitat.

Once you are done writing your plan, use the studio to [build your package](/docs/developing-packages/#plan-builds).

### Related Resources

- [Write plans](/docs/developing-packages/#write-plans): Describes what a plan is and how to create one.
- [Add configuration to plans](/docs/developing-packages/#add-configuration): Learn how to make your running service configurable by templatizing configuration files in your plan.
- [Binary-only packages](/docs/best-practices/binary-wrapper): Learn how to create packages from software that comes only in binary form, like off-the-shelf or legacy programs.

You may also find the [plan syntax guide](/docs/reference/) useful. It lists the settings, variables, and functions that you can use when creating your plan.

+++
## <a name="add-configuration" id="add-configuration" data-magellan-target="add-configuration" type="anchor">Configuration Templates</a>

Chef Habitat allows you to templatize your application's native configuration files using [Handlebars](http://handlebarsjs.com/) syntax. The following sections describe how to create tunable configuration elements for your application or service.

Template variables, also referred to as tags, are indicated by double curly braces: `{{a_variable}}`. In Chef Habitat, tunable config elements are prefixed with `cfg.` to indicate that the value is user-tunable.

Here's an example of how to make a configuration element user-tunable. Assume that we have a native configuration file named `service.conf`. In `service.conf`, the following configuration element is defined:

```conf
recv_buffer 128
```

We can make this user tunable like this:

```handlebars
recv_buffer {{cfg.recv_buffer}}
```

Chef Habitat can read values that it will use to render the templatized config files in three ways:

1. `default.toml` - Each plan includes a `default.toml` file that specifies the default values to use in the absence of any user provided inputs. These files are written in [TOML](https://github.com/toml-lang/toml), a simple config format.
2. At runtime - Users can alter config at runtime using `hab config apply`. The input for this command also uses the TOML format.
3. Environment variable - At start up, tunable config values can be passed to Chef Habitat using environment variables; this most over-riding way of setting these but require you to restart the supervisor to change them.

Here's what we'd add to our project's `default.toml` file to provide a default value for the `recv_buffer` tunable:

```toml
recv_buffer = 128
```

All templates located in a package's `config` folder are rendered to a config directory, `/hab/svc/<pkg_name>/config`, for the running service. The templates are re-written whenever configuration values change.
The path to this directory is available at build time in the plan as the variable `$pkg_svc_config_path` and available at runtime in templates and hooks as `{{pkg.svc_config_path}}`.

All templates located in a package's `config_install` folder are rendered to a config_install directory, `/hab/svc/<pkg_name>/config_install`. These templates are only accessible to the execution of an `install` hook and any changes to the values referenced by these templates at runtime will not result in re-rendering the template.
The path to this directory is available at build time in the plan as the variable `$pkg_svc_config_install_path` and available at runtime in templates and `install` hooks as `{{pkg.svc_config_install_path}}`.

Chef Habitat not only allows you to use Handlebars-based tunables in your plan, but you can also use both built-in Handlebars helpers as well as Chef Habitat-specific helpers to define your configuration logic. See [Reference](/reference/helpers) for more information.

## <a name="pkg-binds" id="pkg-binds" data-magellan-target="pkg-binds" type="anchor">Runtime Binds and Exports</a>

*Runtime binding* in Chef Habitat refers to the ability for one service group to connect to another, forming a producer-consumer relationship where the consumer service can use the producer service's current configuration in order to configure itself at runtime. When the producer's configuration change, the consumer is notified and can reconfigure itself as needed.

With runtime binding, a consumer service can use a "binding name" of their choosing in their configuration and lifecycle hook templates as a kind of handle to refer to the configuration values they need from the producer service. This name isn't inherently tied to any particular package or service group name. Instead, when the service is run, users associate a service group with that binding name, which gives Chef Habitat all the information it needs to wire the producer and consumer services together.

Let's look at how we set up this relationship in detail.

### Defining the Producer Contract

A producer service defines its contract by "exporting" a subset of its runtime configuration. This is done by defining values in the `pkg_exports` associative array defined in your package's `plan.sh`. For example, a database server named `amnesia` might define the following exports:

```bash
pkg_exports=(
  [port]=network.port
  [ssl-port]=network.ssl.port
)
```

Note that Powershell plans use hashtables where Bash plans use associative arrays. A `plan.ps1` would declare its exports as:

```PS
$pkg_exports=@{
  port="network.port"
  ssl-port="network.ssl.port"
}
```

This will export the runtime values of its `network.port` and `network.ssl.port` configuration entries publicly as `port` and `ssl-port`, respectively. All configuration entries in `pkg_exports` must have a default value in `default.toml`, but the actual exported values will change at runtime to reflect the producer's current configuration. When values change (such as when an operator uses `hab config apply`), the consumer service will be notified that its producer service configuration has changed. We'll see how to use this on the consumer in the sections below.

Producer services export only the *subset* of their configuration that is defined through `pkg_exports` and not the entire thing. Consumer services see only what the producer service exports, and nothing more. This is important, because it means that configuration that must remain secret--such as passwords--are not shared _unless_ they are explicitly defined in `pkg_exports`.

Additionally, the internal structure of the producer's configuration is independent of the exported interface it presents. In our example, `ssl-port` originally comes from a deeply-nested `network.ssl.port` value. However, the exported interface is _flat_, effectively a non-nested set of key-value pairs.

### Defining the Consumer Contract

The consumer service defines a "binding name" as a handle to refer to a service group from which it receives configuration data. However, it must do more than just name the bind, it must also state the configuration values it expects from the service group. Chef Habitat will make sure that whatever service group is bound actually exports the expected values to the consumer service.

As an example, let's say we have an application server, called `session-server`, that needs to connect to a database service, and needs both a "port" and an "ssl-port" in order to make that connection. We can describe this relationship in our `plan.sh` file like so:

```bash
pkg_binds=(
  [database]="port ssl-port"
)
```

Here, `pkg_binds` is an associative array. The key ("database") is the binding name, while the value ("port ssl-port") is a space-delimited list of the exported configuration the binding requires. A consumer can specify multiple binds; each would be an individual entry in this associative array. Judging from this, the producer we described above would be a good candidate for this application server to bind to, because it exports both a "port" and an "ssl-port".

A bound service group may export additional values, but they cannot export less and still satisfy the contract.

Chef Habitat only matches services up at the syntactic, not semantic, level of this contract. If you bind to a service that exports a "port", Chef Habitat only knows that the service exports something called "port"; it could be the port for a PostgreSQL database, or it could be the port of an application server. You will need to ensure that you connect the correct services together; Chef Habitat's binds provide the means by which you express these relationships. You are, however, free to create bind names and export names that are meaningful for you.

#### The Difference Between _pkg\_binds_ and _pkg\_binds\_optional_

In addition to the `pkg_binds` array, Plan authors may also specify `pkg_binds_optional`. It has exactly the same structure as `pkg_binds`, but, as the name implies, these bindings are _optional_; however, it is worth examining exactly what is meant by "optional" in this case.

In order to load a service into the Supervisor, each bind defined in `pkg_binds` *must* be mapped to a service group; if any of these binds are not mapped, then the Supervisor will refuse to load the service.

Binds defined in `pkg_binds_optional`, on the other hand, *may* be mapped when loading a service. If a service group mapping is not defined at load time, the Supervisor will load the service without question. As an extreme example, a service could have no `pkg_binds` entries, and five `pkg_binds_optional` entries; such a service could be loaded with no binds mapped, one bind mapped, all the way to mapping all five binds.

There are several scenarios where optional binds may be useful:

 * A service may have some default functionality which may be overridden at load-time by mapping an optional binding. Perhaps you have some kind of artifact repository service that, in the absence of a "remote-store" bind stores data on the local filesystem. However, if `remote-store` is bound to an appropriate S3 API-compatible service, such as [Minio](https://www.minio.io), it could modify its behavior to store data remotely.

 * A service can be optionally bind to a service to unlock additional features. For example, say you have an application that may run with or without a caching layer. You can model this using an optional bind named (say), "cache". If you wish to run without the caching functionality enabled, you can start the service without specifying a service group mapping for the "cache" bind. Since the bind is optional, it is not needed for Chef Habitat to run your service. However, if you do wish to run with the caching enabled, you can specify a service group mapping, e.g. `hab svc load acme/my-app --bind=cache:redis.prod`. In this scenario, your service's configuration can pull configuration values from the `redis.prod` service group, enabling it to use Redis as a caching layer.

* A service may can optionally bind one of several services; if bind "X" is mapped, operate _this_ way; if "Y" is mapped, operate _that_ way. An application that could use either a Redis backend or a PostgreSQL backend, depending on the deployment scenario, could declare optional "redis" and "postgresql" bindings, and pick which one to map at service load-time. If this is your use case, Chef Habitat does not have a way to encode the fact that "one and only one of these optional bindings should be mapped", so you will have to manage that on your own.

### Service Start-up Behavior

Prior to Chef Habitat 0.56.0, if the service group that you bound to was not present in the Supervisor network census, or had no live members, your service would not start until the group was present with live members. While this can be desirable behavior in some cases, as with running certain legacy applications, it is not always desirable, particularly for modern microservice applications, which should be able to gracefully cope with the absence of their networked dependencies.

With 0.56.0, however, this behavior can be modified using the new runtime service option `--binding-mode`. By setting `--binding-mode=relaxed` when loading a service, that service can start immediately, whether there are any members of a bound service group present or not. (Setting `--binding-mode=strict` will give you the previous, start-only-after-all-bound-groups-are-present behavior. This is also the current default, though `relaxed` will be the eventual default for Chef Habitat 1.0.0.). Such a service should have configuration and lifecycle hook templates written in such a way that the service can remain operational (though perhaps with reduced functionality) when there are no live members of a bound service group present in the network census.

#### The Difference Between Required Binds, Optional Binds, and Binding Mode

While there is a bit of overlap in these concepts, they are distinct. It's best to think of required and optional binds as defining "how applications can be wired together" (specifically, which "wires" must be connected in order to provide the minimal amount of information needed to run a service). Binding mode, on the other hand, defines how the application's start-up behavior is affected the presence or absence of its networked dependencies.

Another useful thing to keep in mind when thinking about required and optional binds is that service group mappings currently cannot be dynamically changed at runtime. They can only be changed by stopping a service, reloading the service with a new set of options, and then starting it up again. This constraint (which may change in future versions of Chef Habitat) may help guide your choice between what should be a required bind, and what should be optional, particularly when using the relaxed binding mode.

### Using Runtime Binds with Consumer Services

Once you've defined both ends of the contract, you can leverage the bind in any of your package's hooks or configuration files. Given the two example services above, a section of a configuration file for `session-server` might look like this:

```handlebars
{{~#each bind.database.members as |member|}}
  database = "{{member.sys.ip}}:{{member.cfg.port}}"
  database-secure = "{{member.sys.ip}}:{{member.cfg.ssl-port}}"
{{~/each}}
```

Here, `bind.<BINDING_NAME>` will be "truthy" (and can thus be used in boolean expressions) only if the bind has been satisfied, and `bind.<BINDING_NAME>.members` will be an array of only active members.

(Prior to Chef Habitat 0.56.0, `bind.<BINDING_NAME>` was always present, and `bind.<BINDING_NAME>.members` had _all_ members, even ones that had left the Supervisor network long ago. This necessitated using the `eachAlive` helper function, instead of just `each`.)

### Starting a Consumer Service

Since your application server defined `database` as a required bind, you'll need to provide the name of a service group running a package which fulfills the contract using the `--bind` parameter to the Supervisor. For example, running the following:

```bash
$ hab svc load <ORIGIN>/<NAME> --bind database:amnesia.default
```

would create a bind aliasing `database` to the `amnesia` service in the `default` service group.

The service group passed to `--bind database:{service}.{group}` doesn't *need* to be the service `amnesia`. This bind can be any service as long as they export a configuration key for `port` and `ssl-port`.

You can declare bindings to multiple service groups in your templates by using the `--bind` option multiple times on the command line. Your service will not start if your package has declared a required bind and a value for it was not specified by `--bind`.

+++
## <a name="plan-builds" id="plan-builds" data-magellan-target="plan-builds" type="anchor">Plan Builds</a>

Packages need to be signed with a private origin key at buildtime. Generate an origin key pair manually by running the following command on your host machine:

```bash
$ hab origin key generate <ORIGIN>
```

The `hab-origin` subcommand will place the origin key files, originname-_timestamp_.sig.key (the private key) and originname-_timestamp_.pub files (the public key), in the `$HOME/.hab/cache/keys` directory. If you're creating origin keys in the Studio container, or you are running as root on a Linux machine, your keys will be stored in `/hab/cache/keys`.

Because the private key is used to sign your artifact, it should not be shared freely; however, if anyone wants to download and use your artifact, then they must have your public key (.pub) installed in their local `$HOME/.hab/cache/keys` or `/hab/cache/keys` directory. If the origin's public key is not present, Chef Habitat attempts to download it from the Builder endpoint specified by the `--url` argument (https://bldr.habitat.sh by default) to `hab pkg install`.

### Passing Origin Keys into the Studio

The Habitat Studio is a self-contained and minimal environment, which means that you'll need to share your private origin keys with the Studio to sign artifacts. You can do this in three ways:

1. Set `HAB_ORIGIN` to the name of the origin you intend to use before entering the Studio:

  ```bash
  export HAB_ORIGIN=originname
  ```

  This approach overrides the `HAB_ORIGIN` environment variable and imports your public and private origin keys into the Studio environment. It also overrides any `pkg_origin` values in the packages that you build. This is useful because you can use it to build your own artifact, as well as to build your own artifacts from other packages' source code, for example, `originname/node` or `originname/glibc`.

1. Set `HAB_ORIGIN_KEYS` to the names of your origins. If you're using more than one origin, separate them with commas:

  ```bash
  export HAB_ORIGIN_KEYS=originname-internal,originname-test,originname
  ```

  This imports the private origin keys, which must exactly match the origin names for the plans you intend to build.

1. Use the `-k` flag (short for "keys") which accepts one or more key names separated by commas with:

  ```bash
  hab studio -k originname-internal,originname-test enter
  ```

  This imports the private origin keys, which must exactly match the origin names for the plans you intend to build.

After you create or receive your private origin key, you can start up the Studio and build your artifact.

### Interactive Build

Any build that you perform from a Chef Habitat Studio is an interactive build. Studio interactive builds allow you to examine the build environment before, during, and after the build.

The directory where your plan is located is known as the plan context.

1. Change to the parent directory of the plan context.
1. Create and enter a new Chef Habitat Studio. If you have defined an origin and origin key during `hab cli setup` or by explicitly setting the `HAB_ORIGIN` and `HAB_ORIGIN_KEYS` environment variables, then type the following:

    ```bash
    $ hab studio enter
    ```

    The directory you were in is now mounted as `/src` inside the Studio. By default, a Supervisor runs in the background for iterative testing. You can see the streaming output by running <code>sup-log</code>. Type <code>Ctrl-C</code> to exit the streaming output and <code>sup-term</code> to terminate the background Supervisor. If you terminate the background Supervisor, then running <code>sup-run</code> will restart it along with every service that was previously loaded. You have to explicitly run <code>hab svc unload origin/package</code> to remove a package from the "loaded" list.

3. Enter the following command to create the package.

    ```studio
    $ build /src/planname
    ```

4. If the package builds successfully, it is placed into a `results` directory at the same level as your plan.

#### Managing the Studio Type (Docker/Linux/Windows)

Depending on the platform of your host and your Docker configuration, the behavior of `hab studio enter` may vary. Here is the default behavior listed by host platform:

* **Linux** - A local chrooted Linux Studio. You can force a Docker based studio by adding the `-D` flag to the `hab studio enter` command.
* **Mac** - A Docker container based Linux Studio
* **Windows** - A local Windows studio. You can force a Docker based studio by adding the `-D` flag to the `hab studio enter` command. The platform of the spawned container depends on the mode your Docker service is running, which can be toggled between Linux Containers and Windows Containers. Make sure your Docker service is running in the correct mode for the kind of studio you wish to enter.

> Note: For more details related to Windows containers see [Running Chef Habitat Windows Containers](/docs/best-practices/running-habitat-windows-containers).

#### Building Dependent Plans in the Studio

Writing plans for multiple packages that are dependent on each other can prove cumbersome when using multiple studios, as you need update dependencies frequently. On the other hand, using a single studio allows you to quickly test your changes by using locally built packages. To do so, you should use a folder structure like this:

```bash
$ tree projects
projects/
├── project-a
└── project-b
```

This way, you can `hab studio enter` in `projects/`. If `project-b` depends on `project-a`, you can call `build project-a && build project-b` for example.

### Non-interactive Build

A non-interactive build is one in which Chef Habitat creates a Studio for you, builds the package inside it, and then destroys the Studio, leaving the resulting `.hart` on your computer. Use a non-interactive build when you are sure the build will succeed, or in conjunction with a continuous integration system.

1. Change to the parent directory of the plan context.
1. Build the artifact in an unattended fashion, passing the name of the origin key to the command.

    ```bash
    $ hab pkg build yourpackage -k yourname
    ```

    > Similar to the `hab studio enter` command above, the type of studio where the build runs is determined by your host platform and `hab pkg build` takes the same `-D` flag to force a Docker environment if desired.

1. The resulting artifact is inside a directory called `results`, along with any build logs and a build report (`last_build.env`) that includes machine-parsable metadata about the build.

By default, the Studio is reset to a clean state after the package is built; however, *if you are using the Linux version of `hab`*, you can reuse a previous Studio when building your package by specifying the `-R` option when calling the `hab pkg build` subcommand.

For information on the contents of an installed package, see [Package Contents](/docs/reference/package-contents).

+++
## <a name="debug-builds" id="debug-builds" data-magellan-target="debug-builds" type="anchor">Troubleshooting Builds</a>

### Bash Plans: `attach`

While working on plans, you may wish to stop the build and inspect the environment at any point during a build phase (e.g. download, build, unpack, etc.). In Bash-based plans, Chef Habitat provides an `attach` function for use in your plan.sh that functions like a debugging breakpoint and provides an easy <acronym title="Read, Evaluation, Print Loop">REPL</acronym> at that point. For PowerShell-based plans, you can use the PowerShell built-in `Set-PSBreakpoint` cmdlet prior to running your build.

To use `attach`, insert it into your plan at the point where you would like to use it, e.g.

```bash
 do_build() {
   attach
   make
 }
```

Now, perform a [build](/docs/developing-packages/#plan-builds) -- we recommend using an interactive studio so you do not need to set up the environment from scratch for every build.

```bash
$ hab studio enter
```

```studio
$ build yourapp
```

The build system will proceed until the point where the `attach` function is invoked, and then drop you into a limited shell:

```studio
### Attaching to debugging session
From: /src/yourapp/plan.sh @ line 15 :

    5: pkg_maintainer="The Chef Habitat Maintainers <humans@habitat.sh>"
    6: pkg_source=http://download.yourapp.io/releases/${pkg_name}-${pkg_version}.tar.gz
    7: pkg_shasum=c2a791c4ea3bb7268795c45c6321fa5abcc24457178373e6a6e3be6372737f23
    8: pkg_bin_dirs=(bin)
    9: pkg_build_deps=(core/make core/gcc)
    10: pkg_deps=(core/glibc)
    11: pkg_exports=(
    12:   [port]=srv.port
    13: )
    14:
    15: do_build() {
 => 16:   attach
    17:   make
    18: }

[1] yourapp(do_build)>
```

You can use basic Linux commands like `ls` in this environment. You can also use the `help` command the Chef Habitat build system provides in this context to see what other functions can help you debug the plan.

```studio
[1] yourapp(do_build)> help
Help
  help          Show a list of command or information about a specific command.

Context
  whereami      Show the code surrounding the current context
                (add a number to increase the lines of context).

Environment
  vars          Prints all the environment variables that are currently in scope.

Navigating
  exit          Pop to the previous context.
  exit-program  End the /hab/pkgs/core/hab-plan-build/0.6.0/20160604180818/bin/hab-plan-build program.

Aliases
  @             Alias for `whereami`.
  quit          Alias for `exit`.
  quit-program  Alias for `exit-program`.
```

  Type `quit` when you are done with the debugger, and the remainder of the build will continue. If you wish to abort the build entirely, type `quit-program`.

### PowerShell Plans: `Set-PSBreakpoint`

While there is no `attach` function exposed in a `plan.ps1` file, one can use the native Powershell cmdlet `Set-PSBreakpoint` to access virtually the same functionality. Instead of adding `attach` to your `Invoke-Build` function, enter the following from inside your studio shell:

```studio
[HAB-STUDIO] Habitat:\src> Set-PSBreakpoint -Command Invoke-Build
```

Now upon running `build` you should enter an interactive prompt inside the context of the Invoke-Build function:

```studio
   habitat-aspnet-sample: Building
Entering debug mode. Use h or ? for help.

Hit Command breakpoint on 'Invoke-Build'

At C:\src\habitat\plan.ps1:26 char:23
+ function Invoke-Build {
+                       ~
[HAB-STUDIO] C:\hab\cache\src\habitat-aspnet-sample-0.2.0>>
```

You can now call Powershell commands to inspect variables (like `Get-ChildItem variable:\`) or files to debug your build.


## <a name="pkg-exports" id="pkg-exports" data-magellan-target="pkg-exports" type="anchor">Package Export Formats</a>

You can export packages into several different external, immutable runtime formats. This topic will be updated as more formats are supported in the future. Currently there are exports for: docker, mesos, tar, and cloudfoundry.

The command to export a package is `hab pkg export <FORMAT> <PKG_IDENT>`. See the [Chef Habitat CLI Reference Guide](/docs/habitat-cli#hab-pkg-export) for more CLI information.

> **Note** If you specify an <code>origin/package</code> identifier, such as <code>core/postgresql</code>, the Chef Habitat CLI will check Builder for the latest stable version of the package and export that.

> If you wish to export a package that is not on Builder, create a Chef Habitat artifact by running the `build` command, then point `hab pkg` to the `.hart` file within the `/results` directory:
   ```bash
   hab pkg export tar ./results/example-app.hart
   ```

Read on for more detailed instructions.

### Exporting to Docker

You can create a Docker container image for any package by performing the following steps:

1. Ensure you have a Docker daemon running on your host system. On Linux, the exporter shares the Docker socket (`unix:///var/run/docker.sock`) into the studio.

1. Create an interactive studio with the `hab studio enter` command.

1. [Build](/docs/developing-packages/#plan-builds) the Chef Habitat package from which you want to create a Docker container image and then run the Docker exporter on the package.

    ```bash
    $ hab pkg export docker ./results/<hart-filename>.hart
    ```

    > **Note** The command above is for local testing only. If you have uploaded your package to Builder, you can export it by calling <code>hab pkg export docker origin/package</code>. The default is to use the latest stable release; however, you can override that by specifying a different channel in an optional flag.

    > **Note** On Linux, exporting your Chef Habitat artifact to a Docker image requires the Docker Engine supplied by Docker. Packages from distribution-specific or otherwise alternative providers are currently not supported.

    > **Note** In a Windows container studio, the `export` command will not be able to access the host docker engine. To export a Windows package or hart file built inside of a Windows container studio, first exit the studio and then export the `.hart` file in your local `results` directory.

1. You may now exit the studio. The new Docker image exists on your computer and can be examined with `docker images` or run with `docker run`.

1. Please note that when you run this docker container, you will need to pass the `HAB_LICENSE` environment variable into the container in order to accept the Habitat license. If you don't, your container will abort at a license acceptance prompt. One way to do this would be `docker run --env HAB_LICENSE=accept-no-persist IMAGE`. Alternatively, if you use a scheduler to run these docker containers, you should add that environment variable to your scheduler configuration.

### Exporting to a Tarball

1. Enter the Chef Habitat studio by using `hab studio enter`.

2. Install or [build](/docs/developing-packages/#plan-builds) the Chef Habitat package from which you want to create a tarball, for example:

    ```bash
    $ hab pkg install <ORIGIN>/<NAME>
    ```

3. Run the tar exporter on the package.

    ```bash
    $ hab pkg export tar <ORIGIN>/<NAME>
    ```

    If you receive an error, try running

    ```bash
    $ hab pkg export tar /results/<your_package>.hart
    ```

4. Your package is now in a tar file that exists locally on your computer in the format `<ORIGIN>-<NAME>-<VERSION>-<TIMESTAMP>.tar.gz` and can be deployed and run on a target machine.

5. If you wish to run this tar file on a remote machine (i.e. a virtual machine in a cloud environment), scp (or whatever transfer protocol you prefer) the file to whatever you wish to run it.

6. SSH into the virtual machine

7. Run these commands to set up the required user and group:

    ```bash
    $ sudo adduser --group hab
    $ sudo useradd -g hab hab
    ```

8. Next, unpack the tar file:

    ```bash
    $ sudo tar xf your-origin-package-version-timestamp.tar.gz
    $ sudo cp -R hab /hab
    ```

9. Now, start the Supervisor and load your service package using the `hab` binary, which is included in the tar archive:

    ```bash
    $ sudo /hab/bin/hab sup run
    $ sudo /hab/bin/hab svc load <ORIGIN>/<NAME>
    ```
### Exporting to Kubernetes

The Kubernetes exporter is an additional command line subcommand to the standard Chef Habitat CLI interface. It leverages the existing Docker image export functionality and, additionally, generates a Kubernetes manifest that can be deployed to a Kubernetes cluster running the Chef Habitat operator.

1. Create an interactive studio in any directory with the `hab studio enter` command.

2. Install or [build](/docs/developing-packages/#plan-builds) the Chef Habitat package from which you want to create an application, for example:

    ```bash
    $ hab pkg install <ORIGIN>/<NAME>
    ```

3. Run the Kubernetes exporter on the package.

    ```bash
    $ hab pkg export kubernetes ./results/<hart-filename>.hart
    ```

    You can run `hab pkg export kubernetes --help` to see the full list of available options and general help.

4. The Kubernetes exporter outputs a Kubernetes manifest yaml file. You can redirect the output to a file like this:

    ```bash
    $ hab pkg export kubernetes ./results/<hart-filename>.hart -o my_app.yaml
    ```
5. To push the Docker image created by the Kubernetes exporter to Docker Hub or another container registry, use:

    ```bash
    $ hab pkg export kubernetes --push-image --username <your_docker_hub_username> --password <your_docker_hub_password> -o my_app.yaml
    ```
6. Add the HAB_LICENSE environment variable to the generated manifest YAML file. For example, add the environmental variable directly to the generated manifest:

    ```yaml
    +++
    apiVersion: habitat.sh/v1beta1
    kind: Habitat
    customVersion: v1beta2
    metadata:
    ## Name of the Kubernetes resource.
    name: sample-node-app-1-1-0-20190516204636
    spec:
    v1beta2:
        ## Name of the Habitat service package exported as a Docker image.
        image: user/sample-node-app:1.1.0-20190516204636
        ## Number of desired instances.
        count: 1
        ## An object containing parameters that affects how the Habitat service
        ## is executed.
        service:
        ## Name of the Habitat service.
        name: sample-node-app
        ## Habitat topology of the service.
        topology: standalone
        env:
        - name: HAB_LICENSE
          value: accept-no-persist
    ```

7. You can run this manifest in your Kubernetes cluster with:

    ```bash
    $ kubectl create -f my_app.yaml
    ```

8. This will create a Kubernetes StatefulSet running your package. To access the pod running your package from the outside internet, you will need to add a Kubernetes service (i.e. a Kubernetes load balancer) with an external IP. Here is an example.

    ```bash
    +++
    apiVersion: v1
    kind: Service
    metadata:
    name: app-0
    spec:
    type: LoadBalancer
    selector:
        habitat-name: sample-node-app-1-1-0-20190516204636
    ports:
    - protocol: TCP
        port: 8000
        targetPort: 8000
    ```

9. You can add this service to your Kubernetes cluster with:

    ```bash
    kubectl create -f ./service.yml
    ```

### Export to a Helm Chart

The Helm exporter is an additional command line subcommand to the standard Chef Habitat CLI interface. It is very similar to the Kubernetes exporter but it takes you even further. It also leverages the existing Docker image export functionality but unlike the Kubernetes exporter, instead of generating a Kubernetes manifest, it creates a distributable Helm chart directory. This chart directory can not only be deployed in your local Kubernetes cluster, but also easily packaged and distributed.

Additionally, the Kubernetes Chef Habitat operator is automatically added to the Helm chart as a dependency and hence installed automatically as part of the Chef Habitat Helm chart.

1. Create an interactive studio in any directory with the `hab studio enter` command.

2. Install or [build](/docs/developing-packages/#plan-builds) the Chef Habitat package from which you want to create an application, for example:

    ```bash
    $ hab pkg install <ORIGIN>/<NAME>
    ```

3. Run the Helm exporter on the package.

    ```bash
    $ hab pkg export helm <ORIGIN>/<NAME>
    ```

    You can run `hab pkg export helm --help` to see the full list of available options and general help.

4. More information on how to setup Helm and use of the Helm exporter can be found on the [announcement blog](https://www.habitat.sh/blog/2018/02/Habitat-Helm/)

5. Add the HAB_LICENSE environment variable to the generated chart, usually in the `templates/habitat.yaml` file.

### Exporting to Apache Mesos and DC/OS

1. Create an interactive studio in any directory with the `hab studio enter` command.

2. Install or [build](/docs/developing-packages/#plan-builds) the Chef Habitat package from which you want to create a Marathon application, for example:

    ```bash
    $ hab pkg install <ORIGIN>/<NAME>
    ```

3. Run the Mesos exporter on the package.

    ```bash
    $ hab pkg export mesos <ORIGIN>/<NAME>
    ```

4. This will create a Mesos container-format tarball in the results directory, and also print the JSON needed to load the application into Marathon. Note that the tarball needs to be uploaded to a download location and the "uris" in the JSON need to be updated manually. This is an example of the output:

    ```json
    { "id": "yourorigin/yourpackage", "cmd": "/bin/id -u hab &>/dev/null || /sbin/useradd hab; /bin/chown -R hab:hab *;
    mount -t proc proc proc/; mount -t sysfs sys sys/;mount -o bind /dev dev/; /usr/sbin/chroot . ./init.sh start
    yourorigin/yourpackage", "cpus": 0.5, "disk": 0, "mem": 256, "instances": 1, "uris":
    ["https://storage.googleapis.com/mesos-habitat/yourorigin/yourpackage-0.0.1-20160611121519.tgz" ] }
    ```

5. Note that the default resource allocation for the application is very small: 0.5 units of CPU, no disk, one instance, and 256MB of memory. To change these resource allocations, pass different values to the Mesos exporter as command line options (defaults are documented with `--help`).

6. See the article [Apaches Mesos and DC/OS](/docs/best-practices/mesos-dcos) for more information on getting your application running on Mesos.

### Exporting to Cloud Foundry

Packages can be exported to run in a [Cloud Foundry platform](https://www.cloudfoundry.org/certified-platforms/) through the use of a Docker image that contains additional layers meant to handle mapping from the Cloud Foundry environment to a Chef Habitat default.toml file.

#### Setting up Docker Support in Cloud Foundry

If you have not done so already, you must enable Docker support for Cloud Foundry before you can upload your Cloud Foundry-specific Docker image.

To do so, make sure you have done the following:

1. Log in as an Admin user.
2. Enable Docker support on your Cloud Foundry deployment by enabling the `diego_docker` feature flag.

   ```bash
   $ cf enable-feature-flag diego_docker
   ```

#### Creating a Mapping File

The mapping file is a TOML file that can add Bash-interpolated variables and scripts. The Bash code will have access to:

* all environment variables
* the jq binary
* the helper methods listed below

Here's an example of a mapping TOML file named `cf-mapping.toml`:

```toml cf-mapping.toml
secret_key_base = "$SECRET_KEY_BASE"
rails_env = "$RAILS_ENV"
port = ${PORT}

[db]
user = "$(service "elephantsql" '.credentials.username')"
password = "$(service "elephantsql" '.credentials.password')"
host = "$(service "elephantsql" '.credentials.host')"
name = "$(service "elephantsql" '.credentials.database')"
```

#### Helpers

The helper methods are designed to extract information from the standard Cloud Foundry environment variables [VCAP_SERVICES](https://docs.cloudfoundry.org/devguide/deploy-apps/environment-variable.html#VCAP-SERVICES) and [VCAP_APPLICATION](https://docs.cloudfoundry.org/devguide/deploy-apps/environment-variable.html#VCAP-APPLICATION).

* `service <service-name> <jq-expression>` will extract the JSON associated with the given service-name from the `VCAP_SERVICES` environment variable and apply the jq-expression to it.
* `application <jq-expression>` will apply the jq-expression to the `VCAP_APPLICATION` environment variable

### Exporting and Pushing to a Cloud Foundry Endpoint

1. Create a mapping.toml file using the format specified above and place that file in your local project repo.

2. Enter the Studio through `hab studio enter`.

3. Install or [build](/docs/developing-packages/#plan-builds) the package that you want to export.

    ```bash
    $ hab pkg install <ORIGIN>/<NAME>
    ```

4. Run the Cloud Foundry exporter on the package.

    ```bash
    $ hab pkg export cf <ORIGIN>/<NAME> /path/to/mapping.toml
    ```

   > **Note** To generate this image, a base Docker image is also created. The Cloud Foundry version of the docker image will have `cf-` as a prefix in the image tag.

5. (Optional) If you are creating a web app that binds to another Cloud Foundry service, such as ElephantSQL, you must have this service enabled in your deployment before running your app.

6. [Upload your Docker image to a supported registry](https://docs.cloudfoundry.org/devguide/deploy-apps/push-docker.html). Your Docker repository should be match the `origin/package` identifier of your package.

    ```bash
    $ docker push origin/package:cf-version-release
    ```

7. After your Cloud Foundry Docker image is built, you can deploy it to a Cloud Foundry platform.

    ```bash
    $cf push cf push APP-NAME --docker-image docker_org/repository
    ```

   Your application will start after it has been successfully uploaded and deployed.