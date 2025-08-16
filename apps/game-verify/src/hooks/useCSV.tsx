import { useCallback } from 'react';

const useCSV = () => {
  const convertToCSV = useCallback((jsonArray: any[]) => {
    if (!jsonArray || jsonArray.length === 0) return '';
    
    const headers = Object.keys(jsonArray[0]).join(',');
    const rows = jsonArray.map(obj =>
      Object.values(obj).map(value =>
        `"${value}"` // 用双引号包裹值，防止逗号或换行符干扰
      ).join(',')
    ).join('\n');

    return `${headers}\n${rows}`;
  }, []);

  const downloadCSV = useCallback((csvString: string, filename: string) => {
    const blob = new Blob([csvString], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');

    if (link.download !== undefined) {
      const url = URL.createObjectURL(blob);
      link.setAttribute('href', url);
      link.setAttribute('download', filename);
      link.style.visibility = 'hidden';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
  }, []);

  const downloadJSONAsCSV = useCallback((jsonArray: any[], filename: string) => {
    const csvString = convertToCSV(jsonArray);
    downloadCSV(csvString, filename);
  }, [convertToCSV, downloadCSV]);

  return {
    convertToCSV,
    downloadCSV,
    downloadJSONAsCSV // 这个组合方法可以直接下载JSON数据为CSV文件
  };
};

export default useCSV;