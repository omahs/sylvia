# Sylvia Framework

Sylvia is the old name meaning Spirit of The Wood.

Sylvia is the Roman goddess of the forest.

Sylvia is also a framework created to give you the abstraction-focused and
scalable solution for building your CosmWasm Smart Contracts. Find your way
into the forest of Cosmos ecosystem. We provide you with the toolset, so instead
of focusing on the raw structure of your contract, you can create them in proper
and idiomatic Rust and then just let cargo make sure that they are sound.

Learn more about sylvia in [the book](https://cosmwasm.github.io/sylvia-book/index.html)

## The approach

[CosmWasm](https://cosmwasm.com/) ecosystem core provides the base building
blocks for smart contracts - the
[cosmwasm-std](https://crates.io/crates/cosmwasm-std) for basic CW bindings, the
[cw-storage-plus](https://crates.io/crates/cw-storage-plus) for easier state management,
and the [cw-multi-test](https://crates.io/crates/cw-multi-test) for testing them.
Sylvia framework is built on top of them, so for creating contracts, you don't
have to think about message structure, how their API is (de)serialized, or how
to handle message dispatching. Instead, the API of your contract is a set of
traits you implement on your SC type. The framework generates things like entry
point structures, functions dispatching the messages, or even helpers for multitest.
It allows for better control of interfaces, including validating their completeness
in compile time.

Also, as a side effect, as Sylvia has all the knowledge about the contract API structure,
it can generate many helpers - utilities for multitests or even queriers.

## Using in contracts

First you need your contract crate, which should be a library crate:

```shell
$ cargo new --lib ./my-crate
     Created library `./my-crate` package

```

To use sylvia in the contract, you need to add couple dependencies - sylvia itself,
and additionally: `serde`, `cosmwasm-schema`, `schemars` and `cosmwasm_std`.

```shell
$ cargo add sylvia cosmwasm-schema schemars cosmwasm-std serde
...
```

You should also make sure your crate is compiling as `cdylib`, setting the proper
crate type in `Cargo.toml`. I also like to add `rlib` there, so it is possible to
use the contract as the dependency. Example `Cargo.toml`:

```toml
[package]
name = "my-crate"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
cosmwasm-schema = "1.2.5"
cosmwasm-std = "1.2.5"
schemars = "0.8.12"
serde = "1.0.160"
sylvia = "0.5.0"
```

To build your contract as wasm you can use:

```rust
$ cargo build --target wasm32-unknown-unknown
...
```

## Contract type

In Sylvia, we define our contracts as structures:

```rust
pub struct MyContract;
```

The next step is to create an instantiation message for the contract we have:

```rust
use sylvia::contract;
use sylvia::types::InstantiateCtx;
use sylvia::cw_std::{StdResult, Response};

#[contract]
impl MyContract {
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}
```

This immediately creates the InstantiateMessage type in the same module you created
a contract struct. It looks like this:

```rust
struct InstantiateMsg {}
```

There are no fields there at this point, but they will be when we need them.

For now, we need this message to create a contract instantiate entry point for CosmWasm.
We can achieve it using another macro `entry_point`

```rust
use sylvia::{contract, entry_points;
use sylvia::types::InstantiateCtx;
use sylvia::cw_std::{StdResult, Response};

#[entry_points]
#[contract]
impl MyContract {
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}
```

This will generate for us `instantiate`, `execute` and `query` entry points.
Inside they will call `dispatch` on the msg received and run proper logic defined for the sent
variant of the message.

```rust
pub mod entry_points {
    use super::*;

    #[sylvia::cw_std::entry_point]
    pub fn instantiate(
        deps: sylvia::cw_std::DepsMut,
        env: sylvia::cw_std::Env,
        info: sylvia::cw_std::MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<sylvia::cw_std::Response, StdError> {
        msg.dispatch(&MyContract::new(), (deps, env, info))
            .map_err(Into::into)
    }
```

Now we would like to do something useful in the contract instantiation. Let's
start using the [cw-storage-plus](https://docs.rs/cw-storage-plus/1.0.1/cw_storage_plus/)
to add state to the contract (remember to add it as dependency):

```rust
use cw_storage_plus::Item;

struct MyContract<'a> {
    pub counter: Item<'a, u64>,
}

#[entry_points]
#[contract]
impl MyContract<'_> {
    pub fn new() -> Self {
        Self {
            counter: Item::new("counter")
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
        self.counter.save(ctx.deps.storage, &0)?;

        Ok(Response::new())
    }
}
```

We need to add this generic lifetime because of an optimization in storage plus -
it doesn't want to take an owned string, as we often pass there a static string,
but it also doesn't want to fix the `'static` ownership. 99% of the time, you can
get away with passing just `'static` as the first `Item` generic argument, but I
find it more convenient to introduce this "proxy" lifetime passing everywhere. I
eliminate it in the `new` constructor, where I create the storage-plus accessors
giving them proper keys.

Now let's pass the initial counter state as a function argument:

```rust
#[contract]
impl MyContract<'_> {
    #[msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx, counter: u64) -> StdResult<Response> {
        self.counter.save(ctx.deps.storage, &counter)?;

        Ok(Response::new())
    }
}
```

Sylvia would add the field into the instantiation message, which now becomes this:

```rust
struct InstantiateMsg {
    counter: u64,
}
```

What is essential - the field in the `InstantiateMsg` gets the same name as the
function argument.

Now let's add an execution message to the contract:

```rust
#[contract]
impl MyContract<'_> {
    #[msg(exec)]
    pub fn increment(&self, ctx: ExecCtx) -> StdResult<Response> {
        let counter = self.counter.load(ctx.deps.storage)?;
        self.counter.save(ctx.deps.storage, &(counter + 1))?;
        Ok(Response::new())
    }
}
```

Sylvia generated two message types from this:

```rust
enum ExecMsg {
    Increment {}
}

enum ContractExecMsg {
    MyContract(ExecMsg)
}
```

The `ExecMsg` is the primary one you may use to send messages to the contract.
The `ContractExecMsg` is only an additional abstraction layer that would matter
later when we define traits for our contract.
Thanks to `entry_point` macro it is already being used in the generated entry point and we don't
have to do it manually.

One problem you might face now is that we use the `StdResult` for our contract,
but we often want to define the custom error type for our contracts - fortunately,
it is very easy to do:

```rust
use sylvia::cw_std::ensure;

#[contract]
#[error(ContractError)]
impl MyContract<'_> {
    #[msg(exec)]
    pub fn increment(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let counter = self.counter.load(ctx.deps.storage)?;

        ensure!(counter < 10, ContractError::LimitReached);

        self.counter.save(ctx.deps.storage, &(counter + 1))?;
        Ok(Response::new())
    }
}
```

ContractError here is any error type you define for the contract - most typically
with the [thiserror](https://docs.rs/thiserror/1.0.40/thiserror/) crate.
The error type in an entry points will be updated automatically.

Finally, let's take a look at defining the query message:

```rust
use cosmwasm_schema::cw_serde;
use sylvia::types::QueryCtx;

#[cw_serde]
pub struct CounterResp {
    pub counter: u64,
}

#[contract]
#[error(ContractError)]
impl MyContract<'_> {
    #[msg(query)]
    pub fn counter(&self, ctx: QueryCtx) -> StdResult<CounterResp> {
        self
            .counter
            .load(ctx.deps.storage)
            .map(|counter| CounterResp { counter })
    }
}
```

What you might notice - we can still use `StdResult` (so `StdError`) if we don't
need `ContractError` in a particular function. What is important is that the returned
result type has to implement `Into<ContractError>`, where `ContractError` is a contract
error type - it will all be commonized in the generated dispatching function (so
entry points have to return `ContractError` as its error variant).

Messages equivalent to execution messages are generated.
Again entry point is already generated like in case of execute and instantiate.

## Interfaces

One of the fundamental ideas of Sylvia's framework are interfaces, allowing the
grouping of messages into their semantical groups. Let's define a Sylvia interface:

```rust
pub mod group {
    use super::*;
    use sylvia::interface;
    use sylvia::types::ExecCtx;
    use sylvia::cw_std::StdError;

    #[cw_serde]
    pub struct IsMemberResp {
        pub is_member: bool,
    }

    #[interface]
    pub trait Group {
        type Error: From<StdError>;

        #[msg(exec)]
        fn add_member(&self, ctx: ExecCtx, member: String) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn is_member(&self, ctx: QueryCtx, member: String) -> Result<IsMemberResp, Self::Error>;
    }
}
```

Then we need to implement the trait on the contract type:

```rust
use sylvia::cw_std::{Empty, Addr};
use cw_storage_plus::{Map, Item};

pub struct MyContract<'a> {
    counter: Item<'a, u64>,
    // New field added - remember to initialize it in `new`
    members: Map<'a, &'a Addr, Empty>,
}

#[contract]
#[messages(group as Group)]
impl group::Group for MyContract<'_> {
    type Error = ContractError;

    #[msg(exec)]
    fn add_member(&self, ctx: ExecCtx, member: String) -> Result<Response, ContractError> {
        let member = ctx.deps.api.addr_validate(&member)?;
        self.members.save(ctx.deps.storage, &member, &Empty {})?;
        Ok(Response::new())
    }

    #[msg(query)]
    fn is_member(&self, ctx: QueryCtx, member: String) -> Result<group::IsMemberResp, ContractError> {
        let is_member = self.members.has(ctx.deps.storage, &Addr::unchecked(&member));
        let resp = group::IsMemberResp {
            is_member,
        };

        Ok(resp)
    }
}

#[contract]
#[messages(group as Group)]
impl MyContract<'_> {
    // Nothing changed here
}
```

Here are a couple of things to talk about.

First, note that I defined the interface trait in its separate module with a name
matching the trait name, but written "snake_case" instead of CamelCase. Here I have
`group` module for the `Group` trait, but the `CrossStaking` trait should be placed
in its own `cross_staking` module (note the underscore). This is a requirement right
now - Sylvia generates all the messages and boilerplate in this module and will try
to access them through this module.

Then there is the `Error` type embedded in the trait - it is also needed there,
and the trait bound here has to be at least `From<StdError>`, as Sylvia might
generate code returning an `StdError` in deserialization/dispatching implementation.
The trait can be more strict - this is the minimum.

Another thing to remember is that the `#[msg(...)]` attributes become part of the
function signature - they must be the same for the trait and later implementation.

Finally, every implementation block has an additional
`#[messages(module as Identifier)]` attribute. Sylvia needs it to generate the dispatching
properly - there is the limitation that every macro has access only to its local
scope. In particular - we cannot see all traits implemented by a type and their
implementation from the `#[contract]` crate.

To solve this issue, we put this `#[messages(...)]` attribute pointing to Sylvia
what is the module name where the interface is defined, and giving a unique name
for this interface (it would be used in generated code to provide proper enum variant).

The impl-block with trait implementation also contains the `#[messages]` attribute,
but only one - the one with info about the trait being implemented.

## Macro attributes

`Sylvia` work with multiple attributes. I will explain here how and when to use which of them.

```rust
#[contract(module=contract_module::inner_module)]
impl Interface for MyContract {
...
}
```

`module` is meant to be used when implementing interface on the contract. It's purpose
is to inform `sylvia` where is the contract defined. If the contract is implemented in the same
scope this attribute can and should be omitted.

```rust
#[entry_point]
#[contract]
#[error(ContractError)]
impl MyContract {
...
}
```

`error` is used by both `contract` and `entry_point` macros. It is neccessary in case a custom
error is being used by your contract. If omitted generated code will use `StdError`.

```rust
#[contract]
#[messages(interface as Interface)]
impl MyContract {
...
}

#[contract]
#[messages(interface as Interface)]
impl Interface for MyContract {
...
}
```

`messages` is the attribute for the `contract` macro. We can use it both when implementing contract
and when implementing an interface on a contract. It's purpose is to point sylvia to what interface
is being implemented and how module in which it is defined is called.

In case of the implementation of a trait it is only needed if the trait is defined in different
module. Otherwise it should be omitted.
For the contract implementation it is mandatory for the functionality of an implemented trait
to be part of a contract logic.
For the interface implementation there should be at most one `messages` attribute used.
In case of the contract implementation there can be multiple `messages` attributes used.

`sv::override_entry_point` - refer to `Override entry points` section.

```rust
struct MyMsg;
impl CustomMsg for MyMsg {}

struct MyQuery;
impl CustomQuery for MyMsg {}

#[contract]
#[sv::custom(msg=MyMsg, query=MyQuery)]
impl MyContract {
...
}
```

`sv::custom` allows to define CustomMsg and CustomQuery for the contract. By default generated code
will return `Response<Empty>` and will use `Deps<Empty>` and `DepsMut<Empty>`.

## Single module per macro

Generated items and namespaces may overlap and it is suggested to split all macro calls
into separate modules.
This could also improve the project readability as it would end up split between semantical parts
and save maintainers from possible adjustment in case of new features being introduced in the
future.

## Usage in external crates

What is important is the possibility of using generated code in the external code.
First, let's start with generating the documentation of the crate:

```sh
cargo doc --document-private-items --open
```

This generates and opens documentation of the crate, including all generated structures.
`--document-private-item` is optional, but it will generate documentation of not-public
modules which is sometimes useful.

Going through the doc, you will see that all messages are generated in their structs/traits
modules. To send messages to the contract, we can just use them:

```rust
use sylvia::cw_std::{WasmMsg, to_binary};

fn some_handler(my_contract_addr: String) -> StdResult<Response> {
    let msg = my_contract_crate::ExecMsg::Increment {};
    let msg = WasmMsg::ExecMsg {
        contract_addr: my_contract_addr,
        msg: to_binary(&msg)?,
        funds: vec![],
    }

    let resp = Response::new()
        .add_message(msg);
    Ok(resp)
}
```

We can use messages from traits in a similar way:

```rust
let msg = my_contract_crate::group::QueryMsg::IsMember {
    member: addr,
};

let is_member: my_contract_crate::group::IsMemberResp =
    deps.querier.query_wasm_smart(my_contract_addr, &msg)?;
```

It is important not to confuse the generated `ContractExecMsg/ContractQueryMsg`
with `ExecMsg/QueryMsg` - the former is generated only for contract, not for interfaces,
and is not meant to use to send messages to the contract - their purpose is for proper
messages dispatching only, and should not be used besides the entry points.

## Query helpers

To make querying more user friendly `Sylvia` generates `BoundQuerier` and `Remote` helpers.
The latter is meant to store the address of some remote contract. It's generated implementation
looks like this:

```rust
#[derive(sylvia::serde::Serialize, sylvia::serde::Deserialize)]
pub struct Remote<'a>(std::borrow::Cow<'a, sylvia::cw_std::Addr>);

impl Remote<'_> {
    pub fn querier<'a, C: sylvia::cw_std::CustomQuery>(
        &'a self,
        querier: &'a sylvia::cw_std::QuerierWrapper<'a, C>,
    ) -> BoundQuerier<'a, C> {
        BoundQuerier {
            contract: &self.0,
            querier,
        }
    }
}
```

It has a single method implemented called querier which returns the `BoundQuerier` for the stored
address.

```rust

pub struct BoundQuerier<'a, C: sylvia::cw_std::CustomQuery> {
    contract: &'a sylvia::cw_std::Addr,
    querier: &'a sylvia::cw_std::QuerierWrapper<'a, C>,
}

impl<'a, C: sylvia::cw_std::CustomQuery> Querier for BoundQuerier<'a, C> {
    fn counter(&self) -> Result<CounterResp, sylvia::cw_std::StdError> {
        let query = QueryMsg::counter();
        self.querier.query_wasm_smart(self.contract, &query)
    }
}

pub trait Querier {
    fn counter(&self) -> Result<CounterResp, sylvia::cw_std::StdError>;
}
```

For each query method in the contract `Sylvia` will implement via generated `Querier` trait
method for more user friendly querying.

Let's modify the query from the previous paragraph. Currently it will look as follows:

```rust
let is_member = Remote::new(remote_addr)
    .querier(&ctx.deps.querier)
    .is_member(addr)?;
```

Your contract might be implemented such it will be communicating with some other contract regularly.
In such case you might want to store it as a field in your Contract:

```rust
pub struct MyContract<'a> {
    counter: Item<'a, u64>,
    members: Map<'a, &'a Addr, Empty>,
    // Added
    remote: Item<'a, Remote<'static>>,
}

#[msg(exec)]
pub fn evaluate_member(&self, ctx: ExecCtx, ...) -> StdResult<Response> {
    let is_member = self
        .remote
        .load(ctx.deps.storage)?
        .querier(&ctx.deps.querier)
        .is_member(addr)?;
}
```

`Remote` and `BoundQuerier` types are also generated for the interfaces and you can use them too.
Also using the implemented `From` trait you can convert from `contract::BoundQuerier` to
`interface::BoundQuerier`.

```rust
let remote = self.remote.load(ctx.deps.storage)?;
let querier = remote.querier(&ctx.deps.querier);
let other_count = BoundQuerier::from(&querier).count()?.count;
```

## Using not implemented entry points

Sylvia is not yet implementing all the possible CosmWasm entry points, and even
when it will - it might happen that some will be added in the future, and Sylvia
would not align immediately. Hopefully, you can always use traditional entry points
for anything which is not implemented - for example, IBC calls. As an example, let's
see how to implement replies for messages:

```rust
use sylvia::cw_std::{DepsMut, Env, Reply, Response};

#[contract]
#[entry_point]
#[error(ContractError)]
#[messages(group as Group)]
impl MyContract<'_> {
    fn reply(&self, deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
        todo!()
    }
    // Some items defined previously
}

#[entry_point]
fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    &MyContract::new().reply(deps, env, reply)
}
```

It is important to create an entry function in the contract type - this way, it
gains access to all the state accessors defined on the type.

## Overriding entry points

If above approach is not working for you because f.e. you want to use `sudo` entry point
and generated `multitest helpers` don't yet support it and you are unable to test your contract
or you prefer to use some custom defined entry point it is possible to override the entry point
on the contract.

Let's consider following code:

```rust
#[cw_serde]
pub enum UserExecMsg {
    IncreaseByOne {},
}

pub fn increase_by_one(ctx: ExecCtx) -> StdResult<Response> {
    crate::COUNTER.update(ctx.deps.storage, |count| -> Result<u32, StdError> {
        Ok(count + 1)
    })?;
    Ok(Response::new())
}

#[cw_serde]
pub enum CustomExecMsg {
    ContractExec(crate::ContractExecMsg),
    CustomExec(UserExecMsg),
}

impl CustomExecMsg {
    pub fn dispatch(self, ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        match self {
            CustomExecMsg::ContractExec(msg) => {
                msg.dispatch(&crate::contract::Contract::new(), ctx)
            }
            CustomExecMsg::CustomExec(_) => increase_by_one(ctx.into()),
        }
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CustomExecMsg,
) -> StdResult<Response> {
    msg.dispatch((deps, env, info))
}
```

It is possible to define some custom `exec` message which will dispatch over one generated
by your Contract and one defined by you. To use this custom entry point with `contract` macro
you can add the `sv::override_entry_point(...)` attribute.

```rust    
#[contract]
#[sv::override_entry_point(exec=crate::entry_points::execute(crate::exec::CustomExecMsg))]
#[sv::override_entry_point(sudo=crate::entry_points::sudo(crate::SudoMsg))]
impl Contract {
```

It is possible to override all message types like that. Next to the entry point path you will
also have to provide the type of your custom message. It is required to deserialize the messsage
in the `multitest helpers`.

## Multitest

Sylvia also generates some helpers for testing contracts - it is hidden behind the
`mt` feature flag, which has to be enabled.

It is important to ensure no `mt` flag is set when the contract is built in `wasm`
target because of some dependencies it uses, which are not buildable on Wasm. My
recommendation is to add an additional `sylvia` entry with `mt` enabled in the
`dev-dependencies`, and also add the `mt` feature on your contract, which enables
mt utilities in other contract tests. An example `Cargo.toml`:

```rust
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
library = []
mt = ["sylvia/mt"]

[dependencies]
cosmwasm-schema = "1.2.5"
cosmwasm-std = "1.2.5"
cw-storage-plus = "1.0.1"
schemars = "0.8.12"
serde = "1.0.160"
sylvia = "0.5.0"
thiserror = "1.0.40"

[dev-dependencies]
sylvia = { path = "0.5.0", features = ["mt"] }
```

There would obviously be more dependencies - most probably `cw-storage-plus`,
but this is just to show how I enable the `mt` flag. With that, we can use mt
utils in the contract:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sylvia::multitest::App;

    #[test]
    fn counter_test() {
        let app = App::default();

        let owner = "owner";

        let code_id = contract::CodeId::store_code(&app);

        let contract = code_id.instantiate(3)
            .with_label("My contract")
            .call(owner)
            .unwrap();

        let counter = contract.counter().unwrap();
        assert_eq!(counter, contract::CounterResp { counter: 3});

        contract.increment().call(owner).unwrap();

        let counter = contract.counter().unwrap();
        assert_eq!(counter, contract::CounterResp { counter: 4});
    }
}
```

First of all, note the `contract` module I am using here - it is a slight change
that doesn't match the previous code - I assume here that all the contract code
sits in the `contract` module to make sure it is clear where the used type lies.
So if I use `contract::something`, it is `something` in the module of the original
contract (most probably sylvia-generated).

First of all - we do not use `cw-multi-test` app directly. Instead we use the `sylvia`
wrapper over it. It contains the original multi-test App internally, but it does
it in an internally-mutable manner which makes it possible to avoid passing it
everywhere around. It adds some overhead, but it should not matter for testing code.

We are first using the `CodeId` type generated for every single Sylvia contract
separately. Its purpose is to abstract storing the contract in the blockchain. It
makes sure to create the contract object and pass it to the multitest.

A contract's `CodeId` type has one particularly interesting function - the `instantiate`,
which calls an instantiation function. It takes the same arguments as an instantiation
function in the contract, except for the context that Sylvia's utilities would provide.

The function doesn't instantiate contract immediately - instead, it returns what
is called `InstantiationProxy`. We decided that we don't want to force users to set
all the metadata - admin, label, and funds to send with every instantiation call,
as in the vast majority of cases, they are irrelevant. Instead, the
`InstantiationProxy` provides `with_label`, `with_funds`, and `with_amin` functions,
which set those meta fields in the builder pattern style.

When the instantiation is ready, we call the `call` function, passing the message
sender - we could add another `with_sender` function, but we decided that as the
sender has to be passed every single time, we can save some keystrokes on that.

The thing is similar when it comes to execution messages. The biggest difference
is that we don't call it on the `CodeId`, but on instantiated contracts instead.
We also have fewer fields to set on that - the proxy for execution provides only
the `with_funds` function.

All the instantiation and execution functions return the
`Result<cw_multi_test::AppResponse, ContractError>` type, where `ContractError`
is an error type of the contract.

## Interface items in multitest

Because of implementation restrictions, calling methods from the contract interface
looks slightly different:

```rust
use contract::multitest_utils::Group;

#[test]
fn member_test() {
    let app = App::default();

    let owner = "owner";
    let member = "john";

    let code_id = contract::multitest_utils::CodeId::store_code(&app);

    let contract = code_id.instantiate(0)
        .with_label("My contract")
        .call(owner);

    contract
        .group_proxy()
        .add_member(member.to_owned())
        .call(owner);

    let resp = contract
        .group_proxy()
        .is_member(member.to_owned())

    assert_eq!(resp, group::IsMemberResp { is_member: true });
}
```

Note an additional `group_proxy()` call for executions and queries - it returns an
extra proxy wrapper that would send the messages from a particular interface. I also
had to add trait with group-related methods - it is named in the same way as the
original `Group` trait, but lies in `multitest_utils` module of the contract.

## CustomQuery and CustomMsg

Interfaces can be defined to work with some `CustomQuery`/`CustomMsg`.
Having some messages defined as below:

```rust
struct MyMsg;
impl CustomMsg for MyMsg {}

struct MyQuery;
impl CustomQuery for MyMsg {}
```

we can either make the interface to work only with specified message type via
`sv::custom(..)` like:

```rust
#[interface]
#[sv::custom(query=MyQuery, msg=MyMsg)]
pub trait SomeInterface {
}

#[contract(module=super)]
#[sv::custom(msg=MyMsg, query=MyQuery)]
impl SomeInterface for crate::MyContract {
}
```

or to allow users of this interface to choose with which message type it should be
used. In such case you can define `ExecC` and `QueryC` associated type in the interface.

With interface defined as such:

```rust
#[interface]
pub trait AssociatedInterface {
    type Error: From<StdError>;
    type ExecC: CustomMsg;
    type QueryC: CustomQuery;
}

#[contract(module=super)]
#[sv::custom(msg=MyMsg)]
impl AssociatedInterface for crate::MyContract {
    type Error = StdError;
    type ExecC = MyMsg;
    type QueryC = MyQuery;

    #[msg(exec)]
    fn associated_exec(&self, _ctx: ExecCtx<Self::QueryC>) -> StdResult<Response<Self::ExecC>> {
        Ok(Response::default())
    }
}
```

In case both associated type and `sv::custom()` attribute are defined `sv::custom()`
will be used to determine `CustomMsg` and/or `CustomQuery`.

## Generating schema

Sylvia is designed to generate all the code which cosmwasm-schema relies on - this
makes it very easy to generate schema for the contract. Just add a `bin/schema.rs`
module, which would be recognized as a binary, and add a simple main function there:

```rust
use cosmwasm_schema::write_api;

use my_contract_crate::contract::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}
```

Unfortunately, because of [a bug](https://github.com/CosmWasm/ts-codegen/issues/103)
in the `ts-codegen`, schemas for Sylvia contracts are  not properly interpreted there.
However, we are working on how to solve this issue regardless of the `ts-codegen`
implementation.

## Road map

Sylvia is in the adoption stage right now, but we are still working on more and more
features for you. Here is a rough roadmap for the incoming months:

- Sudo support - Although you can define your own sudo entry point it is currently
  not supported in generated multitest helpers.
- Replies - Sylvia still needs support for essential CosmWasm messages, which are
  replies. We want to make them smart, so expressing the correlation between send
  message end executed handler is more direct and not hidden in the reply dispatcher.
- Migrations - Another important message we don't support, but the reason is similar
  to replies - we want them to be smart. We want to give you a nice way to provide
  upgrading Api for your contract, which would take care of its versioning.
- IBC - we want to give you a nice IBC Api too! However, expect it to be a
  while - we must first understand the best patterns here.
- Better tooling support - The biggest issue of Sylvia is that code it generates
  is not trivial, and not all the tooling handles it well. We are working on improving
  user experience in that regard.

## Troubleshooting

For more descriptive error messages, consider using the nightly toolchain (add `+nightly`
argument for cargo)

- Missing messages from interface on your contract - You may be missing
  `messages(interface as Interface)` attribute.
- Cannot find type BoundQuerier - your `Contract` is defined in different module than current one.
  Your `impl Interface for Contract` should have the `#[contract(module=path::to::Contract)]`
  invocation.
