
import {
  Routes,
  redirect,
} from "react-router-dom";
import { ConfigProvider, Spin } from 'antd';
import { useUserContext } from './components/UserProvider';
import { useEffect, useMemo, useState } from 'react';
import { NoPermissionRoutes, PermissionRoutes } from './routes';


function App() {
  const { user, isLogin } = useUserContext();

  const [isLogined, setIsLogined] = useState<boolean | null>(null);

  const hasPermission = useMemo<boolean | undefined>(() => {
    if (isLogined == null) {
      return undefined;
    }

    if (isLogined === false) {
      return false;
    }

    if (user == null) {
      return undefined;
    }

    return isLogined;
  }, [user, isLogined]);

  const isLoading = useMemo(() => {
    return hasPermission === undefined;
  }, [hasPermission]);

  useEffect(() => {
    isLogin().then((logined) => {
      setIsLogined(logined);
    });
  }, [user, isLogin]);

  useEffect(() => {
    if (hasPermission === false) {
      redirect('/login');
    } else if (hasPermission === true) {
      redirect('/dashboard');
    }
  }, [hasPermission]);

  const renderLoading = () => {
    return (
      <div className='w-[100vw] h-[100vh] flex justify-center items-center'>
        <Spin />
      </div>
    )
  }

  return (
    <ConfigProvider>
      {
       isLoading ? renderLoading()
      : 
      <Routes>
        {hasPermission ? PermissionRoutes() : NoPermissionRoutes()}
      </Routes>
    }  
    </ConfigProvider>
  );
}

export default App
