import { createContext, useState, useContext, useMemo, useEffect } from "react";
import { getUserInfo, isIIAuthenticated, clearCachedUserInfo, getCachedUserInfo } from '../../common/ic-client';
import type { UserVo } from 'declarations/admin/admin.did';

export type UserContextType = {
  user?: UserVo | null;
  isLogin: () => Promise<boolean>;
  update: () => Promise<void>;
  clear: () => Promise<void>;
};

const UserContext = createContext<UserContextType | null>(null);

function useUserValue() {
  const [userInfo, setUserInfo] = useState<UserVo | null | undefined>(null);

  const updateUserInfo = async () => {
    const isAuthenticated = await isIIAuthenticated();
    let _userInfo;
    if (!isAuthenticated) {
      _userInfo = await getCachedUserInfo();
    } else {
      _userInfo = await getUserInfo();
    }
    setUserInfo(_userInfo);
  };

  useEffect(() => {
    isIIAuthenticated()
      .then((isAuthenticated) => {
        if (isAuthenticated) {
          getUserInfo()
            .then((userInfo) => {
              setUserInfo(userInfo);
            });
        }
      });
  }, [])


  return useMemo(() => ({
    user: userInfo,
    isLogin: () => isIIAuthenticated(),
    update: async () => {
      return updateUserInfo();
    },
    clear: async () => {
      await clearCachedUserInfo();
      setUserInfo(null);
    }
  }), [userInfo]);
}



import { ReactNode } from "react";

interface UserProviderProps {
  children: ReactNode;
}

export const UserProvider = ({ children }: UserProviderProps) => {
  const user = useUserValue();


  return (
    <UserContext.Provider value={user}>
      {children}
    </UserContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export const useUserContext = () => {
  const context = useContext(UserContext);
  if (!context) {
    throw new Error("useUserContext must be used within a UserProvider");
  }
  return context;
};