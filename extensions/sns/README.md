
Use the `dfx sns` subcommands to simulate decentralizing a dapp.

The basic syntax for running `dfx sns` commands is:

``` bash
dfx sns [subcommand] [flag]
```

Depending on the `dfx sns` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx sns` commands, select an appropriate command.

| Command                                                            | Description                                                                                        |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- |
| [`init-config-file validate`](#_dfx_sns_init-config-file_validate) | Checks whether the sns config file is valid.                                                       |
| [`deploy-testflight`](#_dfx_sns_deploy-testflight)                 | Creates a test deployment of the SNS canisters according to the local config.                      |
| [`prepare-canisters`](#_dfx_sns_prepare-canisters)                 | Prepares dapp canister(s) for SNS decentralization by adding NNS root as one of their controllers. |
| [`propose`](#_dfx_sns_propose)                                     | Submits a CreateServiceNervousSystem NNS Proposal.                                                 |
| `help`                                                             | Displays usage information message for a specified subcommand.                                     |

To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx sns validate`, you can run the following command:

``` bash
dfx sns validate --help
```

## dfx sns init-config-file validate

Use the `dfx sns validate` command to verify that an SNS configuration file is well formed.

### Basic usage

``` bash
dfx sns init-config-file validate
```

## dfx sns deploy-testflight

Use the `dfx sns deploy-testflight` command to create a testflight deployment of the SNS canisters according to the local configuration file. A testflight is an sns deployed directly to a local replica or the Internet Computer, skipping the proposal, token swap, and sns-wasm canister. The SNS canisters remain controlled by the developer after deployment. See [the testflight documentation](https://internetcomputer.org/docs/current/developer-docs/daos/sns/testing/testing-on-mainnet) for more details.

### Basic usage

``` bash
dfx sns deploy-testflight --init-config-file /path/to/sns_init.yaml
```

## dfx sns prepare-canisters 

### Basic usage

Use the `dfx sns prepare-canisters` command to easily add and remove NNS Root (r7inp-6aaaa-aaaaa-aaabq-cai) 
as a co-controller of your dapp canister(s). Your dapp canister(s) must be under control of the NNS for
a CreateServiceNervousSystem NNS proposal to properly transfer control to a newly created ServiceNervousSystem.

``` bash
dfx sns prepare-canisters 
```

### Examples

Add NNS Root as a co-controller to a dapp canister controlled by the current dfx user

```
dfx sns prepare-canisters add-nns-root rkp4c-7iaaa-aaaaa-aaaca-cai
dfx sns prepare-canisters add-nns-root rkp4c-7iaaa-aaaaa-aaaca-cai 6zikg-xaaaa-aaaaa-aabhq-cai
```

Remove NNS Root as a co-controller to a dapp canister controlled by the current dfx user

```
dfx sns prepare-canisters remove-nns-root rkp4c-7iaaa-aaaaa-aaaca-cai
dfx sns prepare-canisters remove-nns-root rkp4c-7iaaa-aaaaa-aaaca-cai 6zikg-xaaaa-aaaaa-aabhq-cai
```

## dfx sns propose

Use the `dfx sns propose` command to submit a CreateServiceNervousSystem NNS proposal according to the
local configuration file. The local dfx identity must be able to operate (as a controller or hotkey) 
a staked NNS Neuron to submit an NNS Proposal. 

### Basic usage

``` bash
dfx sns propose
```

### Examples

Submit a proposal using a known NeuronId.

```
dfx sns propose --neuron-id 42 sns_init.yaml 
```

Submit a proposal using the memo chosen during NNS Neuron creation. This is used in conjunction
with the current dfx identity to calculate the ICP Ledger Subaccount that backs the NNS Neuron's 
stake.

```
dfx sns propose --neuron-memo 0 sns_init.yaml 
```

**Test Only:** Submits a proposal using the test NNS neuron that is available on the NNS installed
to the local dfx server. If this flag is used when submitting to mainnet, the request will be rejected.

```
dfx sns propose --test-neuron-proposer sns_init.yaml
```