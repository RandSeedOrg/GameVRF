import BigNumber from "bignumber.js";

export const MULTIPLE_BASE = 1e4;

/** 将倍数转换成带基数的倍数 */
export function multipleToMB(multiple: string): number {
  return new BigNumber(multiple).times(MULTIPLE_BASE).toNumber();
}

/** 将带基数的倍数转换成倍数，可直接用于计算 */
export function mbToMultiple(mb: number): BigNumber {
  return new BigNumber(mb).div(MULTIPLE_BASE);
}


export function renderE4s(mb: number): string {
  return mbToMultiple(mb).toString();
}

/** 将数字转换为百分数 */
export function toPercent(value: number | string): string {
  return `${(Number(value) * 100).toFixed(2)}%`;
}

/** 将数字格式化为千分位表示 */
export function formatNumberWithCommas(value: number | string | bigint): string {
  return new BigNumber(Number(value)).toFormat();
}

const CYCLE_TO_T_BASE = 1e12;

export function cyclesToT(value: number | string | bigint): string {
  return new BigNumber(Number(value)).div(CYCLE_TO_T_BASE).toFixed(6);
}

export function formatCycles(value: number | string | bigint): string {
  return `${cyclesToT(value)}T (${Number(value)})`;
}