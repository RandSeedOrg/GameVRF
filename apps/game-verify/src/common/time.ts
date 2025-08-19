import dayjs from 'dayjs';
import utc from 'dayjs/plugin/utc';
import timezone from 'dayjs/plugin/timezone';

dayjs.extend(utc);
dayjs.extend(timezone);

// 将一个纳秒值转为一个字符串，分别显示小时、分钟、秒
export function formatDurationNanos(time: number | string | bigint): string {
  console.log('time: ', time);
  time = Number(time);
  const hours = Math.floor(time / 3600000000000);
  const minutes = Math.floor((time % 3600000000000) / 60000000000);
  const seconds = Math.floor((time % 60000000000) / 1000000000);
  return `${hours}h ${minutes}m ${seconds}s`;
}

// 将一个纳秒值转为一个数组，分别表示小时、分钟、秒
export function durationNanosToHMS(time: number | string | bigint): [number, number, number] {
  time = Number(time);
  const hours = Math.floor(time / 3600000000000);
  const minutes = Math.floor((time % 3600000000000) / 60000000000);
  const seconds = Math.floor((time % 60000000000) / 1000000000);
  return [hours, minutes, seconds];
}

// 将一个小时、分钟、秒的值转为一个纳秒值
export function hmsToDurationNanos(hours: number | null | undefined, minutes: number | null | undefined, seconds: number | null | undefined): number {
  console.log('hmsToDurationNanos: ', hours, minutes, seconds);
  return (hours || 0) * 3600000000000 + (minutes || 0) * 60000000000 + (seconds || 0) * 1000000000;
}


export function formatTimestampNano(timestamp: number | string | bigint | undefined): string {
  if (!timestamp) {
    return '--';
  }
  
  return dayjs(Number(timestamp) / 1000000).tz('GMT+0').format('YYYY-MM-DD HH:mm:ss');
}

export function formatTimestampSeconds(timestamp: number | string | bigint | undefined): string {
  if (!timestamp) {
    return '--';
  }
  return dayjs(Number(timestamp) * 1000).tz('GMT+0').format('YYYY-MM-DD HH:mm:ss');
}

export function formatTimestampMs(timestamp: number | string | bigint): string {
  return dayjs(Number(timestamp)).tz('GMT+0').format('YYYY-MM-DD HH:mm:ss');
}

// 获取当前时间相对于 GMT+0 的时间差(毫秒)
export function getTimezoneOffsetMs(): number {
  // 使用原生 JS 获取分钟偏移量（UTC+8 返回 -480）
  const offsetMinutes = new Date().getTimezoneOffset();
  // 转换为毫秒（注意符号取反）, UTC+8 返回 480 * 60 * 1000 = 288000000
  return -offsetMinutes * 60 * 1000;
}