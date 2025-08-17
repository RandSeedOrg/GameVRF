import { getDictByDictCode } from "@/common/sys/dict";
import { Tag } from "antd";
import { DictItemVo } from "declarations/admin/admin.did";
import { useEffect, useMemo, useState } from "react";

export type DictLabelProps = {
  dictCode: string;
  value: string;
  colord?: boolean;
}

export function DictLabel({ dictCode, value, colord = false }: DictLabelProps) {
  const [item, setItem] = useState<DictItemVo>();

  const loadDictItem = async (dictCode: string, value: string) => {
    const dict = await getDictByDictCode(dictCode);
    
    if (!dict) {
      return;
    }
    
    setItem(dict.items.find((item) => item.value === value));
  };

  useEffect(() => {
    loadDictItem(dictCode, value);
  }, [dictCode, value]);

  const color = useMemo(() => {
    if (!colord ) {
      return 'green-inverse';
    }

    if (value === '0') {
      return 'red-inverse';
    }
    return 'blue-inverse';
  }, [colord, value]);

  return <Tag color={color}>{item?.label || value}</Tag>;
}