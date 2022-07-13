<h1 align="center"><br>
    Substrate Weight Compare
<br></h1>

<h4 align="center">Compares weight files that where generated by Substrate</h4>

<p align="center">
  <img alt="GitHub" src="https://img.shields.io/github/license/ggwpez/substrate-weight-compare">
  <a href="https://weights.tasty.limo/">
    <img src="https://weights.tasty.limo/version/badge"/>
  </a>
</p>

This project parses and analyzes Substrate weight files. It does so by parsing the rust code. The results can be analyzed in the terminal or in the browser. See the deployment link above for an example.

# Install

Install both binaries:

```sh
cargo install --git https://github.com/ggwpez/substrate-weight-compare swc swc_web
```

# Example: Web Interface

Assuming you have a Substrate compatible repository checked out in the parent directory:

```sh
swc-web --root ../ --repos polkadot substrate cumulus
```

then open your browser and try the following:
- [http://localhost:8080/](http://localhost:8080/)

# Example: Compare weight files

Suppose you have some weight files in:
- `OLD=repos/polkadot/` and
- `NEW=my_other_repos/polkadot`   
The base command looks like this:

```sh
swc compare files --old $OLD/* --new $NEW/* --method worst
```

If you want to compare the weights of the Kusama to the Polkadot runtime, the command becomes a bit more longer:
```sh
swc compare files --old ../polkadot/runtime/kusama/**/weights/*.rs --new ../polkadot/runtime/polkadot/*/weights/*.rs --method worst --ignore-errors --change changed unchanged --unit time --threshold 10
```

```sh
+-----------------------------------------+-----------------------------+----------+----------+---------------+
| File                                    | Extrinsic                   | Old      | New      | Change [%]    |
+=============================================================================================================+
| pallet_election_provider_multi_phase.rs | feasibility_check           | 1.23ms   | 812.80us | -33.90 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| frame_benchmarking_baseline.rs          | addition                    | 162.00ns | 112.00ns | -30.86 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_hrmp.rs              | hrmp_cancel_open_request    | 27.90us  | 39.02us  | +39.86 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| pallet_tips.rs                          | slash_tip                   | 15.86us  | 22.61us  | +42.56 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_initializer.rs       | force_approve               | 3.12us   | 4.53us   | +45.02 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_hrmp.rs              | clean_open_channel_requests | 366.82us | 590.73us | +61.04 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_hrmp.rs              | hrmp_accept_open_channel    | 29.81us  | 48.54us  | +62.81 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_hrmp.rs              | hrmp_close_channel          | 27.58us  | 44.92us  | +62.89 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_hrmp.rs              | force_process_hrmp_open     | 2.17ms   | 3.64ms   | +67.28 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_hrmp.rs              | hrmp_init_open_channel      | 32.67us  | 55.70us  | +70.49 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_hrmp.rs              | force_process_hrmp_close    | 1.21ms   | 2.11ms   | +74.25 |
|-----------------------------------------+-----------------------------+----------+----------+---------------|
| runtime_parachains_hrmp.rs              | force_clean_hrmp            | 1.82ms   | 3.27ms   | +80.04 |
+-----------------------------------------+-----------------------------+----------+----------+---------------+
```
Cou can use the `--print-terms` flag to print the terms. This example omits them since the rows get really long.


# Example: Compare Polkadot Commits

Compare arbitrary Polkadot commits assuming that you have checked the repo out:

```sh
FROM=20467ccea1ae
TO=ef922a7110eb
THRESHOLD=30

# Compare the commits
swc compare commits $FROM $TO --threshold $THRESHOLD --repo ../polkadot

pallet_scheduler.rs::on_initialize_named_aborted 4957 -> 3406 ns (-31.29 %)
pallet_election_provider_multi_phase.rs::finalize_signed_phase_reject_solution 33389 -> 19348 ns (-42.05 %)
pallet_election_provider_multi_phase.rs::submit 77368 -> 42754 ns (-44.74 %)
pallet_election_provider_multi_phase.rs::on_initialize_nothing 23878 -> 12324 ns (-48.39 %)
pallet_election_provider_multi_phase.rs::finalize_signed_phase_accept_solution 50596 -> 25888 ns (-48.83 %)
pallet_scheduler.rs::on_initialize_resolved 3886 -> 1701 ns (-56.23 %)
pallet_election_provider_multi_phase.rs::on_initialize_open_unsigned 33568 -> 12320 ns (-63.30 %)
pallet_election_provider_multi_phase.rs::on_initialize_open_signed 34547 -> 12500 ns (-63.82 %)
pallet_election_provider_multi_phase.rs::create_snapshot_internal 8835233 -> 47360 ns (-99.46 %)
pallet_scheduler.rs::on_initialize_periodic_resolved 33 -> 0 ns (-100.00 %)
frame_benchmarking_baseline.rs::subtraction 100 -> 131 ns (+31.00 %)
frame_benchmarking_baseline.rs::addition 96 -> 126 ns (+31.25 %)
pallet_scheduler.rs::on_initialize_periodic 5139 -> 7913 ns (+53.98 %)
runtime_common_crowdloan.rs::on_initialize 0 -> 4293 ns (+100.00 %)
```
It prints first the ones that decreased (good) and then the ones that increased (bad) sorted by ascending absolute value.

# Config options

## Repository

Selects the project to use. *SWC* has the goal of being compatible with:
- [Substrate]
- [Polkadot]
- [Cumulus]

Other projects which are currently compatible, but not a hard requirement:
- [Moonbeam]
- [Composable Finance]

Note: Not all repositories are deployed to the *SWC* web service.

## Path Pattern

Uses the [glob](https://docs.rs/glob/latest/glob/) crate to match files in the repository path with the given pattern.  
Here are some examples for Polkadot:  
- Weight files of all runtimes: `runtime/*/src/weights/**/*.rs`
- All Kusama weight files: `runtime/kusama/src/weights/**/*.rs`

`weights/**/*.rs` is preferred to `weights/*.rs` to include possible sub-folders like XCM.  
The `mod.rs` file is *always* excluded.  

## Pallet

Filter by the pallets to include by using a [Regex].  
Examples:
- `.*` would be *any* pallet.
- `system|assets` would be the `system` and the `assets` pallet.
- `

## Extrinsic

Analogous to the [Pallet](#pallet) filter this filters by the extrinsics using a [Regex].  
Examples:
- `.*` would be *any* extrinsic.
- `mint|burn` would be the `mint` and the `burn` extrinsics.

## Evaluation Method

The evaluation method defines how the weight equation is evaluate (=calculated).  
This is a deciding factor when making a decision whether or not a weight got worse.

- *Base*: Only consider the constant factor of the weight plus storage operations.
- *Guess Worst*: This sets all components to 100 and thereby emulating high inputs. This is a best-effort approach in case your weight files do not have [component range annotations](https://github.com/paritytech/substrate/issues/11397).
- *Exact Worst*: Calculates the exact worst case weight by setting all components to their respective maximum. This requires your weight files to support component range annotations. One way to check that is to search for the string `"The range of component"` in your weight.rs files.

NOTE: The storage weights are currently set to RocksDB Substrate default.  
This will be changed to include the correct values soon.
## Rel Threshold

Filters the changes results by an absolute percentual threshold.  
The percentages values are calculated as increase or decrease.  
Eg: from 100 to 150 would be +50% and would be included by any threshold >=50.

## Abs Threshold

Filters the changes results by an absolute threshold.

## Unit

Controls the unit of the output. Can be set to:
- Weight: Raw weight numbers (default).
- Time: Time each extrinsic took.

## Ignore Errors

Silently ignore parse errors. This is useful when using inclusive path patterns.

## Cache

The web UI caches success responses for 10 minutes. Currently there is no flag to disable it.  
Use commit hashes instead of tags and branches if you need uncached results.

# Running the Tests

## Integration tests

The test use the Polkadot repo in `repo/Polkadot`.

```sh
git clone https://github.com/ggwpez/substrate-weight-compare
cd substrate-weight-compare
git clone https://github.com/paritytech/polkadot/ repos/polkadot
cargo test --release --all-features
```

<!-- LINKS -->
[Substrate]: https://github.com/paritytech/substrate
[Polkadot]: https://github.com/paritytech/polkadot
[Cumulus]: https://github.com/paritytech/cumulus
[Moonbeam]: https://github.com/PureStake/moonbeam
[Composable Finance]: https://github.com/ComposableFi/composable
[Regex]: https://github.com/fancy-regex/fancy-regex
