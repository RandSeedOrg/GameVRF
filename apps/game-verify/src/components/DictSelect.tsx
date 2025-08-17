import { getDictByDictCode } from "@/common/sys/dict";
import { RefSelectProps, Select } from "antd";
import { forwardRef, useEffect, useState } from "react";

export type DictSelectProps = {
  dictCode: string;
  placeholder?: string;
  value?: string;
  disabled?: boolean;
  className?: string;
  allowClear?: boolean;
  onChange?: (value: string) => void;

}

export const DictSelect = forwardRef<RefSelectProps, DictSelectProps>(({ className, allowClear = false, disabled = false, dictCode, placeholder = 'Please Select', value, onChange }: DictSelectProps, ref) => {
  const [options, setOptions] = useState<{ value: string, label: string }[]>([]);

  const loadDictOptions = async (dictCode: string) => {
    const dict = await getDictByDictCode(dictCode);

    console.log('dict: ', dict);

    if (!dict) {
      setOptions([]);
      return;
    }
    setOptions(dict.items);
  };

  useEffect(() => {
    loadDictOptions(dictCode);
  }, [dictCode]);

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