# Rancher Project Mapper Policy

## Introduction

This repository contains the source for a Kubewarden policy to automatically annotation/label namespaces to join preexisting Rancher projects.

This is useful to ensure namespaces are scoped to specifically predefined Rancher RBAC controls.

Settings for the policy are as follows (matches are top-to-bottom ordered in the 'projects' list):
```json
{
  "cluster": "c-m-abc12345",
  "projects": [
    {
      "match_type": "exact",
      "project_name": "p-abc12",
      "namespace_match": "foobar"
    },
    {
      "match_type": "prefix",
      "project_name": "p-def34",
      "namespace_match": "dev-"
    },
    {
      "match_type": "regex",
      "project_name": "p-ghi56",
      "namespace_match": "dev-"
    }, 
  ]
}
```

## Installation

### Prerequisites

1. Install [Cert Manager](https://cert-manager.io/docs/installation/) onto your cluster.

2. Install [Kubewarden](https://docs.kubewarden.io/quick-start) onto your cluster.

### Installation via Helm

1. Check outh the [values.yaml](./chart/values.yaml) to see configurable values, especially around the projects.

2. Create a `/tmp/values.yaml` locally and override with your specific project structure (and any other customizations):
  ```yaml
  settings:
    clusterName: c-m-abcdef12 // Name of the cluster within Rancher ('.status.clusterName' in the cluster resource)
    projects:
    - match_type: exact // Can be 'exact', 'regex', or 'prefix'
      project_name: p-abc12 // Name of the project in Rancher to map to ('name' in the project resource)
      namespace_match: foobar // Namespace match string to compare to downstream namespaces upon creation
  ```

3. Install the Kubewarden policy via helm:
  ```bash
  helm install -n kubewarden -f /tmp/values.yaml rancher-project-mapper-policy/chart/
  ``` 

## Code organization

The code that takes care of parsing the settings can be found inside of the
`settings.go` file.

The actual validation code is defined inside of the `validate.go` file.

The `main.go` contains only the code which registers the entry points of the
policy.

## Implementation details

> **DISCLAIMER:** WebAssembly is a constantly evolving topic. This document
> describes the status of the Go ecosystem at April 2021.

Currently the official Go compiler cannot produce WebAssembly binaries
that can be run **outside** of the browser. Because of that, Kubewarden Go
policies can be built only with the [TinyGo](https://tinygo.org/) compiler.

TinyGo doesn't yet support all the Go features (see [here](https://tinygo.org/lang-support/)
to see the current project status). Currently its biggest limitation
is the lack of a fully supported `reflect` package. Among other things, that
leads to the inability to use the `encoding/json` package against structures
and user defined types.

Kubewarden policies need to process JSON data like the policy settings and
the actual request received by Kubernetes.
However it's still possible to write a Kubewarden policy by using some 3rd party
libraries.

This is a list of libraries that can be useful when writing a Kubewarden
policy:

* [Kubernetes Go types](https://github.com/kubewarden/k8s-objects) for TinyGo:
  the official Kubernetes Go Types cannot be used with TinyGo. This module provides all the
  Kubernetes Types in a TinyGo-friendly way.
* [easyjson](https://github.com/mailru/easyjson/): this provides a way to
  marshal and unmarshal Go types without using reflection.
* Parsing JSON: queries against JSON documents can be written using the
  [gjson](https://github.com/tidwall/gjson) library. The library features a
  powerful query language that allows quick navigation of JSON documents and
  data retrieval.
* Generic `set` implementation: using [Set](https://en.wikipedia.org/wiki/Set_(abstract_data_type))
  data types can significantly reduce the amount of code inside of a policy,
  see the `union`, `intersection`, `difference`,... operations provided
  by a Set implementation.
  The [mapset](https://github.com/deckarep/golang-set) can be used when writing
  policies.

Last but not least, this policy takes advantage of helper functions provided
by [Kubewarden's Go SDK](https://github.com/kubewarden/policy-sdk-go).

## Testing

This policy comes with a set of unit tests implemented using the Go testing
framework.

As usual, the tests are defined inside of the `_test.go` files. Given these
tests are not part of the final WebAssembly binary, the official Go compiler
can be used to run them.

The unit tests can be run via a simple command:

```shell
make test
```

It's also important the test the final result of the TinyGo compilation:
the actual WebAssembly module.

This is done by a second set of end-to-end tests. These tests use the
`kwctl` cli provided by the Kubewarden project to load and execute
the policy.

The e2e tests are implemented using [bats](https://github.com/bats-core/bats-core):
the Bash Automated Testing System.

The end-to-end tests are defined inside of the `e2e.bats` file and can
be run via this command:

```shell
make e2e-tests
```

## Automation

This project contains the following [GitHub Actions](https://docs.github.com/en/actions):

  * `e2e-tests`: this action builds the WebAssembly policy, installs
    the `bats` utility and then runs the end-to-end test
  * `unit-tests`: this action runs the Go unit tests
  * `release`: this action builds the WebAssembly policy and pushes it to a
    user defined OCI registry ([ghcr](https://ghcr.io) is a perfect candidate)
