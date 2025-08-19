import { durationNanosToHMS, hmsToDurationNanos } from "@/common/time";
import { InputNumber } from "antd";
import { forwardRef, useEffect, useState } from "react";

export type DurationInputProps = {
  value?: number;
  disabled?: boolean;
  maxHours?: number;
  onChange?: (value: number) => void;
}

export const DurationInput = forwardRef<HTMLInputElement, DurationInputProps>(({ disabled = false, value, onChange }: DurationInputProps, ref) => {
  const [hours, setHours] = useState<number | null>();
  const [minutes, setMinutes] = useState<number | null>();
  const [seconds, setSeconds] = useState<number | null>();

  const changeHours = (value: number | null) => {
    onChange?.(hmsToDurationNanos(value, minutes, seconds));
  }
  const changeMinutes = (value: number | null) => {
    onChange?.(hmsToDurationNanos(hours, value, seconds));
  }
  const changeSeconds = (value: number | null) => {
    onChange?.(hmsToDurationNanos(hours, minutes, value));
  }

  useEffect(() => {
    const [h, m, s] = durationNanosToHMS(value || 0);
    setHours(h || undefined);
    setMinutes(m || undefined);
    setSeconds(s || undefined);
  }, [value]);

  return <div  className="flex items-center">
    <InputNumber ref={ref} disabled={disabled} min={0} step={1} value={hours} onChange={changeHours} placeholder="Hours"/><span className="mx-2">h</span>
    <InputNumber disabled={disabled}  min={0} step={1} value={minutes} onChange={changeMinutes} placeholder="Minutes"/><span className="mx-2">m</span>
    <InputNumber disabled={disabled}  min={0} step={1} value={seconds} onChange={changeSeconds} placeholder="Seconds"/><span className="mx-2">s</span>
    </div>;
});