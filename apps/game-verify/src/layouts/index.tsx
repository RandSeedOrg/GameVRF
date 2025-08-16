import React, { useState } from 'react';
import {
  MenuFoldOutlined,
  MenuUnfoldOutlined,
} from '@ant-design/icons';
import { Button, Layout, theme } from 'antd';
import { Outlet } from 'react-router-dom';
import { AppMenu } from './AppMenu';
import { AppHeader } from './AppHeader';

const { Sider, Content } = Layout;

export const AdminLayout: React.FC = () => {
  const [collapsed, setCollapsed] = useState(false);
  
  const {
    token: { colorBgContainer, borderRadiusLG },
  } = theme.useToken();

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <div className='h-[100vh] overflow-auto bg-white'>
        <Sider theme='light' trigger={null} collapsible collapsed={collapsed}>
          <div className="sticky top-0 bg-white z-[100] h-[64px] flex items-center justify-center gap-2 text-black text-2xl">
            <div className="logo"></div>
            {!collapsed && <div>WLMC</div>}
          </div>
          <AppMenu />
        </Sider>
      </div>
      <Layout>
        <AppHeader>
          <Button
            type="text"
            icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
            onClick={() => setCollapsed(!collapsed)}
            style={{
              fontSize: '16px',
              width: 64,
              height: 64,
            }}
          />
        </AppHeader>
        <Content
          style={{
            margin: '24px 16px',
            padding: 24,
            // minHeight: 280,
            height: 'calc(100vh - 120px)',
            overflow: 'auto',
            background: colorBgContainer,
            borderRadius: borderRadiusLG,
          }}
        >
          <Outlet />
        </Content> 
      </Layout>
    </Layout>
  );
};

export default AdminLayout;