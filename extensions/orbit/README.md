
Use the `dfx orbit` subcommands to manage an orbit digital asset manager.

The basic syntax for running `dfx orbit` commands is:

``` bash
dfx sns [subcommand] [flag]
```

Depending on the `dfx sns` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx sns` commands, select an appropriate command.

| Command                                                            | Description                                                                                                        |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| [`login??`]()                                                      | Verify that we can log in to the root oisy canister so that we can list wallets.                                   |
| [`wallet list`](#ss)                                               | Lists the available wallets                                                                                        |
| [`wallet use`](#ss)                                                | Uses the named orbit wallet by default                                                                             |
| [`me`](#me)                                                        | Shows the current user information and permissions for a wallet                                                    |
| [`request list`](#_d)                                              | Lists all requests                                                                                                 |
| [`request vote`](#_d)                                              | Approve or reject a request                                                                                        |
| [`request canister add`](#_d)                                      | Requests that a canister is added to Orbit                                                                         |
| [`request canister install`](#_d)                                  | Requests that a canister wasm is updated or installed                                                              |
| [`request canister assets`](#_d)                                   | Requests that HTTP assets are installed for the given canister                                                     |
| [`request canister call`](#_d)                                     | Requests that an API call is made to a given canister                                                              |



To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx orbit request`, you can run the following command:

``` bash
dfx orbit request --help
```

## `dfx request list`

Use the `dfx request list` command to list all currently open requests.

| Flag | Description |
| --- | --- |
| `--all` | Lists all requests, regardless of state |
| `--state <state>` | Lists requests in a given state.  E.g. `executed`  |
