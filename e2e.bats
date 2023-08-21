#!/usr/bin/env bats

@test "Accept an istio-enabled namespace" {
	run kwctl run  --request-path test_data/namespace-enabled.json  annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
 }

@test "Reject istio-disabled" {
	run kwctl run  --request-path test_data/namespace-disabled.json annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":false.*') -ne 0 ]
	[ $(expr "$output" : '.*is not istio enabled.*') -ne 0 ]
 }
