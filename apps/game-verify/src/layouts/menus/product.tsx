import {
  FundProjectionScreenOutlined,
  DeploymentUnitOutlined,
  ControlOutlined,
} from '@ant-design/icons';

export default [{
  key: '/products',
  icon: <DeploymentUnitOutlined />,
  label: 'Products',
  children: [
    {
      key: '/products/list',
      label: 'List',
      icon: <FundProjectionScreenOutlined />
    },
    {
      key: '/products/rules',
      label: 'Rules',
      icon: <ControlOutlined />,
      children: [
        {
          key: '/products/rules/instant-win',
          label: 'Instant Win',
        },
        {
          key: '/products/rules/daily-4-balls',
          label: 'Daily 4 Balls',
        },
      ]
    },
    {
      key: '/products/instant-win-cycle-logs',
      label: 'Cycle Logs',
      icon: <FundProjectionScreenOutlined />
    },
  ],
}]