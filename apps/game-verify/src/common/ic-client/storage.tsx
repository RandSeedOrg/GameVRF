import { UserVo } from "declarations/admin/admin.did";
import { kvStorage } from "../kv-storage";

const DEVICE_ID = '__device_id__';
const USER_INFO = '__user_info__';
// const USER_INFO_CACHE_LIFETIME = 30 *60 * 1000; // 24小时

/** 设置设备唯一标识 */
export async function getDeviceId() {
  return await kvStorage.getItem(DEVICE_ID) || '';
}

/** 获取设备唯一标识 */
export async function saveDeviceId(deviceId: string) {
  return await kvStorage.setItem(DEVICE_ID, deviceId);
}

/** 缓存用户信息 */
export async function cacheUserInfo(userInfo: UserVo) {
  const deviceId = await getDeviceId();

  if (!deviceId) {
    await saveDeviceId(userInfo.principal_id);
  }

  return await kvStorage.setItem(USER_INFO, userInfo);
}

/** 获取用户信息 */
export async function getCachedUserInfo(): Promise<UserVo | null> {
  const userCache = await kvStorage.getItem(USER_INFO);

  return userCache as UserVo;
}

/** 清理缓存中的用户信息 */
export async function clearCachedUserInfo() {
  return await kvStorage.removeItem(USER_INFO);
}
