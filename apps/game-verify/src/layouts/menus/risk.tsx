import {
  AlertOutlined,
  ToolOutlined,
  StopOutlined,
  TeamOutlined,
  MobileOutlined,
} from '@ant-design/icons';

export default [{
  key: '/risk',
  icon: <AlertOutlined />,
  label: 'Risk',
  children: [
    {
      key: '/risk/config',
      label: 'Risk Config',
      icon: <ToolOutlined />,
    },
    {
      key: '/risk/level',
      label: 'Risk Level',
      icon: <TeamOutlined />,
    },
    {
      key: '/risk/blacklist',
      label: 'Address Blacklist',
      icon: <StopOutlined />,
    },
    {
      key: '/risk/device',
      label: 'Device Blacklist',
      icon: <MobileOutlined />,
    }
  ]
}]