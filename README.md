# frs

frs is a command wrapper tool.

### Install

```shell
cargo install --git https://github.com/Myriad-Dreamin/frs.git
```

### Examples

Before examples:

```shell
alias frx = "frs run --"
```

##### Example 1: Execute with Build Command

Execute the build command using cmake before executing the c++ application.

```shell
myriad-dreamin.iris in ~/work/rust/frs
λ(default)
$ frs with command -- cmake --build cmake-build-debug --parallel 12 --target install
λ(default) :: exec(cmake)
$ frx src/bin/app
# cmake --build cmake-build-debug --parallel 12 --target install
[0/1] Install the project...
-- Install configuration: "Debug"
-- Installing: ./cmake-build-debug/include/absl/base/options.h
-- Installing: ./cmake-build-debug/include/absl/base/options.h
# src/bin/app
Running application...
```

##### Example 2: Execute in docker

Run command inside a container

```shell
myriad-dreamin.iris in ~/work/rust/frs
λ(default)
$ cat /etc/lsb-release /etc/os-release
DISTRIB_ID=Ubuntu
DISTRIB_RELEASE=22.04
...
UBUNTU_CODENAME=jammy
λ(default)
$ frs with docker ubuntu:18.04
λ(default) :: ctr(ubuntu:18.04)
$ frx cat /etc/lsb-release /etc/os-release
DISTRIB_ID=Ubuntu
DISTRIB_RELEASE=18.04
...
UBUNTU_CODENAME=bionic
```

##### Example 3: Execute with workdir and environemnt variables

Execute the command with specific environment variables in a specific working directory.

```shell
myriad-dreamin.iris in ~/work/rust/frs
λ(default)
$ frs with env ASAN_OPTIONS allow_addr2line=1
λ(default) :: env(ASAN_OPTIONS)
$ frs with workdir ~/work/rust/frs/.data/test-workdir
λ(default) :: env(ASAN_OPTIONS) :: workdir(..test-workdir)
$
```

##### Example 4: save and restore context

Save and restore context.

```shell
myriad-dreamin.iris in ~/work/rust/frs
λ(default) :: env(ASAN_OPTIONS)
$ frs save asan_option
λ(asan_option)
$ frs with context another_ctx
λ(another_ctx)
$ frs with context asan_option
λ(asan_option)
$ frs save app --namespace cpp_project
λ(cpp_project::app)
$ frs with context asan_option
λ(asan_option)
$
```

##### Example 5: Dry Run and Inspect Context

Check what's exactly running and the context.

```shell
myriad-dreamin.iris in ~/work/rust/frs
λ(frs_test)
$ frs inspect
# $ run "cmake --build cmake-build-debug --parallel 12 --target install"
# ! core::with_command "cmake --build cmake-build-debug --parallel 12 --target install"
# $ set "FRS_KEY"="test"
# ! core::with_env "FRS_KEY"="test"
# frs_env: FRS_VERSION=0.1.0
(cmake --build cmake-build-debug --parallel 12 --target install;
 (export FRS_KEY=test;
 (((( echo 'frs placeholder' ))))))
λ(frs_test)
$ frs run --show -- echo '$FRS_KEY'
(cmake --build cmake-build-debug --parallel 12 --target install;
 (export FRS_KEY=test;
 echo $FRS_KEY))
```

### Usage

```
The cli for frs.

Usage: frs [COMMAND]

Commands:
  help     Print this message or the help of the given subcommand(s)
  inspect  Inspect context
  run      Run with context
  save     Save context
  with     Manipulate context
```

### Extension

The frs `with` command can be extended using any programming language. To create an frs extension, you must create a program that reads the context either from a file system path or stdin and produces an exact context in JSON format.

```shell
λ(app)
$ frs with ext -- python script/activate.py -- customized_arguments...
λ(app) :: ext(python:activate)
$ python script/activate.py --from-file ~/.confg/frs/context/default/app.json customized_arguments... # the underlying command
```

The program should output a JSON object that represents the context, similar to the following example:

```json
{"meta":{"step_log":[{"description":"core::context \"app\"","prompt":"activate ..\"app\""}]},...}
```
