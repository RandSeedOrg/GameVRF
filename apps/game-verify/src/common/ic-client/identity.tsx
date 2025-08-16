// import { ProjectL_backend as project_backend } from "declarations/ProjectL_backend";
import { icp_ledger_canister } from "./open-canisters/icp_ledger_canister";
import { Actor, Identity } from "@dfinity/agent";
import { AuthClient } from "@dfinity/auth-client";
import { cacheUserInfo, clearCachedUserInfo } from "@/common/ic-client/storage";
import { admin } from "declarations/admin";
import { product_manager } from "declarations/product_manager";
import { instant_win } from "declarations/instant_win"; 
import { marketing } from "declarations/marketing";
import { user } from "declarations/user"
import { pay_center } from "declarations/pay_center";
import { messenger } from "declarations/messenger";


import { message } from 'antd'
import { MILLISECOND, MONTH } from "./time";
import { staking } from "declarations/staking";
import { assets_management } from "declarations/assets_management";

// const backendAgent = Actor.agentOf(project_backend);
const adminAgent = Actor.agentOf(admin);
const productManagerAgent = product_manager && Actor.agentOf(product_manager);
const icpLedgerAgent = Actor.agentOf(icp_ledger_canister);
const instantWinAgent = instant_win && Actor.agentOf(instant_win);
const marketingAgent = marketing && Actor.agentOf(marketing);
const userAgent = user && Actor.agentOf(user);
const payCenterAgent = pay_center && Actor.agentOf(pay_center);
const stakingAgent = staking && Actor.agentOf(staking);
const messengerAgent = Actor.agentOf(messenger);
const assetsManagementAgent = Actor.agentOf(assets_management);

// export function getBackend() {
//   return project_backend;
// }

async function replaceIdentity(authClient: AuthClient & { settedIdentity?: Identity }) {
  const identity = authClient.getIdentity();
  if(authClient.settedIdentity === identity) return;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const expireTime = (identity as unknown as any).getDelegation().delegations[0].delegation.expiration;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  console.log('登录成功，会话有效期至：', (identity as unknown as any).getDelegation().delegations[0].delegation, expireTime, new Date(Number(expireTime / 1000000n)));

  // backendAgent?.replaceIdentity?.(identity);
  adminAgent?.replaceIdentity?.(identity);
  productManagerAgent?.replaceIdentity?.(identity);
  icpLedgerAgent?.replaceIdentity?.(identity);
  instantWinAgent?.replaceIdentity?.(identity);
  marketingAgent?.replaceIdentity?.(identity);
  userAgent?.replaceIdentity?.(identity);
  payCenterAgent?.replaceIdentity?.(identity);
  stakingAgent?.replaceIdentity?.(identity);
  messengerAgent?.replaceIdentity?.(identity);
  assetsManagementAgent?.replaceIdentity?.(identity);
  authClient.settedIdentity = identity;
}

export async function setupIdentity(actorInstance: Actor) {
  const authClient = await getAuthClient();
  const identity = authClient.getIdentity();
  const instanceAgent = Actor.agentOf(actorInstance);
  instanceAgent?.replaceIdentity?.(identity);
}

function onIISessionExpires() {
  console.log('----onExpires----');
}

async function onIILoginSuccess() {
  const authClient = await getAuthClient();
  await replaceIdentity(authClient);

  authClient.idleManager?.registerCallback(onIISessionExpires);
  console.log('----onIILoginSuccess----');
}

let authClient: Promise<AuthClient> | null = null;

async function getAuthClient() {
  if (!authClient) {
    authClient = AuthClient.create({
      idleOptions: {
        idleTimeout: Number(MONTH / MILLISECOND),
        disableDefaultIdleCallback: true,
        onIdle: onIISessionExpires,
      },
    });
  }

  return authClient;
}

export async function isIIAuthenticated() {
  const authClient = await getAuthClient();

  const isAuthenticated = await authClient.isAuthenticated();

  if (isAuthenticated) {
    await replaceIdentity(authClient);
  }

  return isAuthenticated;
}

export async function iiLogin() {
  const authClient = await getAuthClient();
  
  return new Promise((resolve, reject) => {
    authClient.login({
      maxTimeToLive: MONTH,
      // eslint-disable-next-line no-undef
      identityProvider: process.env.DFX_NETWORK === 'local' ? `http://qhbym-qaaaa-aaaaa-aaafq-cai.localhost:8080/` : 'https://identity.ic0.app',
      onSuccess: resolve,
      onError: reject,
    });
  })
  .then(() => {
    return onIILoginSuccess();
  }).catch((err) => {
    console.error('Internet Computer Identity登录失败', err);
    throw err;
  });
}

/** 获取用户信息，如果没有认证登录的话，会强制认证登录 */
export async function getUserInfo() {
  let isAuthenticated = await isIIAuthenticated();
  // const prevAuthenticated = isAuthenticated;

  if(!isAuthenticated){
    // 如果没有认证会话，则进行认证
    await iiLogin();
    isAuthenticated = await isIIAuthenticated();
  }

  // 如果认证成功了，则已替换完agent中的身份信息，接着调用后端接口进行
  if (isAuthenticated) {
    const [userInfo] = await admin.get_current_user().catch((err: Error) => {
      console.log('获取用户信息失败', err.message);
      if (err.message?.indexOf('IcCanisterSignature signature could not be verified: public key') > -1) {
        iiLogout().finally(() => {
          location.reload();
        });
        return [null];
      } else {
        message.error('Get user info failed：' + err.message);
      }
      throw err;
    });

    if (userInfo) {
      console.log('获取用户信息成功: ', userInfo);
      await cacheUserInfo(userInfo);
    } else {
      console.error('获取用户信息失败');
      message.error('Get user info failed');
    }

    return userInfo;
  }

  return null;
}

/** 获取当前用户的Principal */
export async function getPricipal() {
  const isAuthenticated = await isIIAuthenticated();
  if (!isAuthenticated) {
    await getUserInfo();
  }

  const authClient = await getAuthClient();

  return authClient?.getIdentity().getPrincipal();
}


export async function iiLogout() {
  const authClient = await getAuthClient();
  const res = await authClient?.logout();
  await clearCachedUserInfo();
  return res;
}