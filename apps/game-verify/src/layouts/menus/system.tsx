import {
  DesktopOutlined,
  UserOutlined,
  KeyOutlined,
  CrownOutlined,
  ReadOutlined,
} from '@ant-design/icons';

export default [{
  key: '/system',
  icon: <DesktopOutlined />,
  label: 'System',
  children: [
    {
      key: '/system/user',
      label: 'User',
      icon: <UserOutlined />
    },
    {
      key: '/system/role',
      label: 'Role',
      icon: <CrownOutlined />
    },
    {
      key: '/system/permission',
      label: 'Permission',
      icon: <KeyOutlined />
    },
    {
      key: '/system/dict',
      label: 'Dictionary',
      icon: <ReadOutlined />
    }
  ],
}];