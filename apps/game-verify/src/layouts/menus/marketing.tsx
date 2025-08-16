import {
  TagsOutlined,
  GiftOutlined,
  TrophyOutlined,
  NodeIndexOutlined
} from '@ant-design/icons';

export default [{
  key: '/marketing',
  icon: <TagsOutlined />,
  label: 'Marketing',
  children: [
    {
      key: '/marketing/bonus-code',
      label: 'Bonus Code',
      icon: <GiftOutlined />,
    },
    {
      key: '/marketing/points',
      label: 'Points',
      icon: <TrophyOutlined />,
    },
    {
      key: '/marketing/points-redeem',
      label: 'Points Redeem',
      icon: <NodeIndexOutlined />,
    }
  ]
}]