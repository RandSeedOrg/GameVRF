import { admin } from "declarations/admin";
import { DictVo } from "declarations/admin/admin.did";

const dictCache: Record<string, DictVo> = {};  
  
export const getDictByDictCode = async (dictCode: string): Promise<DictVo | null> => {
  let dict: DictVo | undefined = dictCache[dictCode];
  if (!dict) {
    [dict] = await admin.get_dict_with_code(dictCode);
    if (dict) {
      dictCache[dictCode] = dict;
    }
  }
  
  if (!dict) {
    return null;
  }

  return {
    ...dict,
    items: dict.items.map((item) => JSON.parse(JSON.stringify(item))),
  };
};


export const getDictItemLabel = async (dict_code: string, value: string) => {
  const dict = await getDictByDictCode(dict_code);

  if (!dict) {
    return value;
  }

  const item = dict.items.find((item) => item.value === value);
  return item ? item.label : value;
}