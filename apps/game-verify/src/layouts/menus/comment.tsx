import {
  CommentOutlined,
  AlignLeftOutlined,
  AuditOutlined,
  ToolOutlined,
} from '@ant-design/icons';

export default [{
  key: '/comment',
  icon: <CommentOutlined />,
  label: 'Comment',
  children: [
    {
      key: '/comment/comments',
      label: 'Comments',
      icon: <AlignLeftOutlined /> ,
    },
    {
      key: '/comment/audit',
      label: 'Audit',
      icon: <AuditOutlined />,
    },
    {
      key: '/comment/config',
      label: 'Config',
      icon: <ToolOutlined />,
    }
  ]
}]