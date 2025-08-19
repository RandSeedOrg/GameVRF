import { useUserContext } from "@/components/UserProvider";
import { Button } from "antd";
import { LogoutOutlined } from "@ant-design/icons";
import { iiLogout } from "@/common/ic-client";
import { useMemo } from "react";


export function AppUser() {
  const { user, clear } = useUserContext();

  const handleLogout = () => {
    iiLogout().then(() => {
      clear();
    });
  }

  const tips = useMemo(() => {
    if (user?.is_controller) {
      return 'Controller';
    }

    if (!user?.roles?.length) {
      return 'Anonymous';
    }

    return 'Admin';
  }, [user]);

  return (
    <div className='px-4 flex justify-center items-center gap-4'>
      <div className="text-ellipsis overflow-hidden whitespace-nowrap">
        <span>{tips}</span>
        <span className="ml-2 font-semibold">{user?.name || user?.principal_id}</span>
      </div>
      <Button danger type='primary' icon={<LogoutOutlined />} onClick={handleLogout} />
    </div>
  );
}