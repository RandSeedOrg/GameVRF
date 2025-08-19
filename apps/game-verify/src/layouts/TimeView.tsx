
import { formatTimestampMs } from "@/common/time";
import { useEffect, useState } from "react";


export function TimeView() {
  const [time, setTime] = useState(formatTimestampMs(new Date().getTime()));

  useEffect(() => {
    // eslint-disable-next-line no-undef
    let timer: NodeJS.Timeout | null = null;
    
    const updateTime = () => {

      if (timer) {
        clearTimeout(timer);
        timer = null;
      }

      setTime(formatTimestampMs(new Date().getTime()));

      timer = setTimeout(() => {
        updateTime();
      }, 1000);

    };

    updateTime();

    return () => {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
    };
  }
  , []);

  return (
    <div className='px-4 flex justify-center items-center gap-2 font-semibold'>
      {time}
      <span>GMT+0</span>
    </div>
  );
}