use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo};

pub struct MigrateCtx<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
}

pub struct InstantiateCtx<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
}

pub struct ExecCtx<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
}

pub struct QueryCtx<'a> {
    pub deps: Deps<'a>,
    pub env: Env,
}

impl<'a> From<(DepsMut<'a>, Env)> for MigrateCtx<'a> {
    fn from((deps, env): (DepsMut<'a>, Env)) -> Self {
        Self { deps, env }
    }
}

impl<'a> From<(DepsMut<'a>, Env, MessageInfo)> for InstantiateCtx<'a> {
    fn from((deps, env, info): (DepsMut<'a>, Env, MessageInfo)) -> Self {
        Self { deps, env, info }
    }
}

impl<'a> From<(DepsMut<'a>, Env, MessageInfo)> for ExecCtx<'a> {
    fn from((deps, env, info): (DepsMut<'a>, Env, MessageInfo)) -> Self {
        Self { deps, env, info }
    }
}

impl<'a> From<(Deps<'a>, Env)> for QueryCtx<'a> {
    fn from((deps, env): (Deps<'a>, Env)) -> Self {
        Self { deps, env }
    }
}
