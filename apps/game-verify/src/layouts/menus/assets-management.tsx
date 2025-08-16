import {
  MoneyCollectOutlined,
} from '@ant-design/icons';

export default [{
  key: '/assets-management',
  icon: <MoneyCollectOutlined />,
  label: 'Assets Management',
  children: [
    {
      key: '/assets-management/proposal',
      label: 'Proposal',
    },
    {
      key: '/assets-management/address-book',
      label: 'Address Book',
    }
  ],
}];