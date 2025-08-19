import {
  MoneyCollectOutlined,
} from '@ant-design/icons';

export default [{
  key: '/staking',
  icon: <MoneyCollectOutlined />,
  label: 'Staking',
  children: [
    {
      key: '/staking/pool',
      label: 'Staking Pool',
    },
    {
      key: '/staking/account',
      label: 'Staking Account',
    },
    {
      key: '/staking/reward',
      label: 'Staking Reward',
    },
    {
      key: '/staking/event-log',
      label: 'Event Log',
    },
    {
      key: '/staking/subscription',
      label: 'Subscription',
    },
    {
      key: '/staking/transaction',
      label: 'Transaction Record',
    },
  ],
}];