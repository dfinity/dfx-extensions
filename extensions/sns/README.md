
Use the `dfx sns` subcommands to simulate decentralizing a dapp.

The basic syntax for running `dfx sns` commands is:

``` bash
dfx sns [subcommand] [flag]
```

Depending on the `dfx sns` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx sns` commands, select an appropriate command.

| Command                                            | Description                                                    |
|----------------------------------------------------|----------------------------------------------------------------|
| [`create`](#_dfx_sns_create)                       | Creates an SNS configuration template.                         |
| [`validate`](#_dfx_sns_validate)                   | Checks whether the sns config file is valid.                   |
| [`deploy`](#_dfx_sns_deploy)                       | Deploys SNS canisters according to the local config.           |
| [`prepare-canisters`](#_dfx_sns_prepare-canisters) | Prepares dapp canister(s) for SNS decentralization.            |
| [`propose`](#_dfx_sns_propose)                     | Submits a CreateServiceNervousSystem NNS Proposal.             |
| `help`                                             | Displays usage information message for a specified subcommand. |

To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx sns validate`, you can run the following command:

``` bash
dfx sns validate --help
```


## dfx sns create

Use the `dfx sns create` command to create an SNS configuration file. The configuration file specifies important, legally and financially relevant details about dapp decentralization.  The file leaves blank parameters such as token name; you will need to fill these in.

### Basic usage

``` bash
dfx sns create
```

### Examples

You can use the `dfx sns create` command to create and view a configuration file:

``` bash
dfx sns create
less sns.yml
```

## dfx sns validate

Use the `dfx sns validate` command to verify that an SNS configuration file is well formed.

### Basic usage

``` bash
dfx sns validate
```

### Examples

You can use the `dfx sns validate` command to verify that a configuration template is valid.  It is not; it needs details such as token name:

``` bash
dfx sns config create
```
Fill in the blank fields, then:
``` bash
dfx sns config validate
```

## dfx sns deploy

Use the `dfx sns deploy` command to create SNS canisters according to the local configuration file.

Note:  Deploying SNS canisters does not require a proposal, however there is a hefty fee.  Please don't create canisters on mainnet until you have tested your configuration locally and are sure that you are happy with it.

### Basic usage

``` bash
dfx sns deploy
```

### Examples

Create an SNS on the local testnet:
``` bash
dfx sns config create
```
Fill in the blank fields, then:
``` bash
dfx sns config validate
dfx sns deploy
```
You can now verify that the sns canisters have been created.  E.g.:
```
dfx canister info sns_root
dfx canister info sns_ledger
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