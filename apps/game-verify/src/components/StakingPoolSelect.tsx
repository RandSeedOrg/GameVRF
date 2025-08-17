import { e8sToValue } from "@/common/ic-client/utils";
import { RefSelectProps, Select } from "antd";
import { staking } from "declarations/staking";
import { forwardRef, useEffect, useState } from "react";

export type StakingPoolSelectProps = {
  placeholder?: string;
  value?: string;
  disabled?: boolean;
  className?: string;
  allowClear?: boolean;
  onChange?: (value: string) => void;
}

export const StakingPoolSelect = forwardRef<RefSelectProps, StakingPoolSelectProps>(({ className, allowClear = false, disabled = false, placeholder = 'Please Select', value, onChange }: StakingPoolSelectProps, ref) => {
  const [options, setOptions] = useState<{ value: string, label: string }[]>([]);

  const loadStakingPools = async () => {
    const pools = await staking.get_all_staking_pools();

    console.log('pools: ', pools);

    if (!pools) {
      setOptions([]);
      return;
    }
    setOptions(pools.map(pool => ({
      value: pool.id.toString(),
      label: `ID: ${pool.id.toString()}, P: ${e8sToValue(pool.pool_size)}, S: ${e8sToValue(pool.staked_amount)}, A: ${e8sToValue(pool.available_funds)}`,
    })));
  };

  useEffect(() => {
    loadStakingPools();
  }, []);

  return <Select
    className={className}
    ref={ref}
    value={value}
    onChange={onChange}
    placeholder={placeholder}
    disabled={disabled}
    allowClear={allowClear}
    options={options}
  />
});