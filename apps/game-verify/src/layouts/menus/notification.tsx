import {
  BellOutlined,
  ContainerOutlined,
  FieldTimeOutlined,
  MailOutlined
} from '@ant-design/icons';

export default [{
  key: '/notification',
  icon: <BellOutlined />,
  label: 'Message',
  children: [
    {
      key: '/notification/list',
      label: 'Notification List',
      icon: <ContainerOutlined />,
    },
    {
      key: '/notification/history',
      label: 'Delivery List',
      icon: <FieldTimeOutlined />,
    },
    {
      key: '/notification/email-verification',
      label: 'Email Verification',
      icon: <MailOutlined />,
    }
  ]
}]