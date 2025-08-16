import { useEffect, useState } from "react";
import { admin } from "declarations/admin";
import { DictVo } from "declarations/admin/admin.did";
import { Select } from "antd";

const dictCache: Record<string, DictVo> = {};

export function useDictSelect(dictCode: string, placeholder = 'Please select'): {Select: JSX.Element, toLabel: (value: string) => string} {
  const [options, setOptions] = useState<{ value: string, label: string }[]>([]);

  const loadDictOptions = async (dictCode: string) => {
    if (dictCache[dictCode]) {
      setOptions(dictCache[dictCode].items);
      return;
    }
    const [dict] = await admin.get_dict_with_code(dictCode);

    if (!dict) {
      setOptions([]);
      return;
    }
    dictCache[dictCode] = dict;
    setOptions(dict.items);
  };

  useEffect(() => {
    loadDictOptions(dictCode);
  }, [dictCode]);

  return {
    Select: <Select placeholder={placeholder} options={options} />,
    toLabel: (value: string) => {
      const option = options.find((o) => o.value === value);
      return option?.label || value;
    },
  };
}