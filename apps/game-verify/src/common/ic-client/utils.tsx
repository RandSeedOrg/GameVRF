import BigNumber from "bignumber.js";

// 1 ICP = 10^8 e8s
const E8S_BASE = new BigNumber(10).pow(new BigNumber(8));

/** 将e8s转为显示到界面上的 */
export function e8sToIcp(e8s: string | number | bigint | undefined): string {
  if (!e8s) {
    return "0";
  }
  return new BigNumber(e8s.toString(10)).div(E8S_BASE).toString(10);
}

/** 将icp转换成e8s */
export function icpToE8s(amount: string | number) {
  if (!amount) {
    return "0";
  }
  return new BigNumber(amount.toString(10)).times(E8S_BASE).toString(10);
}

export function valueToE8s(value: string | number): bigint {
  return BigInt(icpToE8s(value));
}

export function e8sToValue(value: bigint | string | number | undefined): string {
  return e8sToIcp(value);
}


export function e8sToPercent100Value(value: bigint): string {
  return new BigNumber(value.toString(10)).div(E8S_BASE).times(100).toString(10);
}

export function percent100ValueToE8s(value: string | number): bigint {
  return BigInt(new BigNumber(value.toString(10)).div(100).times(E8S_BASE).toString(10));
}