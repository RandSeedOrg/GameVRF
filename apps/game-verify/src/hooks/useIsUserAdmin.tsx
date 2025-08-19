
import { useState, useEffect } from 'react';
import { user } from 'declarations/user';

const useIsUserAdmin = (): (boolean | null) => {
  const [isAdmin, setIsAdmin] = useState<boolean | null>(null);

  useEffect(() => {
    const checkAdminStatus = async () => {
      try {
        const res = await user.is_admin();
        setIsAdmin(res);
      } catch (error) {
        console.error('Failed to fetch admin status:', error);
        setIsAdmin(false);
      }
    };

    checkAdminStatus();
  }, []);

  return isAdmin;
};

export default useIsUserAdmin;