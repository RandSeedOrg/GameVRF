import { Menu } from 'antd';
import { SelectEventHandler } from 'rc-menu/lib/interface';
import {
  FundProjectionScreenOutlined,
} from '@ant-design/icons';
import { useEffect, useMemo, useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import type { ItemType, MenuItemType } from 'antd/es/menu/interface';
import systemMenus from './menus/system';
import productMenus from './menus/product';
import marketingMenus from './menus/marketing';
import accountMenus from './menus/account';
import userMenus from './menus/user';
import reportMenus from './menus/report';
import riskMenus from './menus/risk';
import stakingMenus from './menus/staking';
import notificationMenus from './menus/notification';
import commentMenus from './menus/comment';
import { useUserContext } from '@/components/UserProvider';
import useIsUserAdmin from '@/hooks/useIsUserAdmin';
import assetsManagement from './menus/assets-management';

export function AppMenu() {
  const navigate = useNavigate();
  const { pathname } = useLocation()

  const { user } = useUserContext();
  const isUserAdmin = useIsUserAdmin();

  const emailRole = user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('messenger::email')))
  const messageMenus = [
    {
      ...notificationMenus[0],
      children: [
        ...notificationMenus[0].children?.filter((menu) => menu.key !== '/notification/email-verification'),
        ...(emailRole ? [notificationMenus[0].children?.find((menu) => menu.key === '/notification/email-verification')] : [])
      ]
    }
  ]

  const items = useMemo<ItemType<MenuItemType>[]>(() => ([
    {
      key: '/dashboard',
      icon: <FundProjectionScreenOutlined />,
      label: 'Dashboard',
    },
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('product::'))) ? productMenus : []),
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('marketing::'))) ? marketingMenus : []),
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('user::'))) ? userMenus : []),
    ...(isUserAdmin ? riskMenus : []),
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('report::'))) ? reportMenus : []),
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('staking::'))) ? stakingMenus : []),
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('assets_management::'))) ? assetsManagement : []),
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('account::'))) ? accountMenus : []),
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('messenger::'))) ? messageMenus : []),
    ...(user?.roles?.some((role) => role?.permissions?.some(permission => permission?.permission_code?.startsWith('messenger::comment::'))) ? commentMenus : []),
    ...(user?.is_controller ? systemMenus : []),
  ]), [user, isUserAdmin]);
  const [openKeys, setOpenKeys] = useState<string[]>([]);
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);

  useEffect(() => {
    const pathname = window.location.pathname;
    const partials = pathname.split('/').map((path) => `/${path}`).slice(1, -1);
    const openKeys = partials.map((path, index) => {
      if (index === 0) {
        return path;
      }
      return partials.slice(0, index + 1).join('');
    });

    setOpenKeys(openKeys);
    setSelectedKeys([pathname]);
  }, []);

  useEffect(() => {
    if (pathname === '/user' || pathname === '/risk/level') {
      setSelectedKeys([pathname]);
    }
  }, [pathname]);

  const onSelect: SelectEventHandler = ({ selectedKeys }) => {
    setSelectedKeys(selectedKeys);
    navigate(selectedKeys[0]);
  };

  const onOpenChange = (openKeys: string[]) => {
    setOpenKeys(openKeys);
  }

  return (
    <Menu
      mode="inline"
      openKeys={openKeys}
      selectedKeys={selectedKeys}
      onSelect={onSelect}
      onOpenChange={onOpenChange}
      items={items}
    />
  );
}