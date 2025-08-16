import { Button, Popover, Space } from "antd";
import { ReactNode, useState } from "react";

export type ConfirmPopoverProps = {
  onConfirm?: () => void;
  title?: string;
  content?: string | ReactNode;
  confirmText?: string;
  cancelText?: string;
  children: ReactNode;
}

export function ConfirmPopover({ onConfirm, title = 'Tip', content = 'Are you sure you want to operate it?', confirmText = 'Ok', cancelText = 'Cancel', children }: ConfirmPopoverProps) {
  const [open, setOpen] = useState(false);

  const handleOpenChange = (open: boolean) => {
    setOpen(open);
  }
  
  return <Popover
    content={<>
      <p>{content}</p>
      <div className="flex items-center justify-end mt-4">
        <Space size="small">
          <Button size="small" onClick={() => setOpen(false)}>{cancelText}</Button>
          <Button size="small" type="primary" onClick={() => { onConfirm?.(); setOpen(false); }}>{confirmText}</Button>
        </Space>
      </div>
    </>}
    title={title}
    trigger="click"
    open={open}
    onOpenChange={handleOpenChange}
  >
    {children}
  </Popover>
}