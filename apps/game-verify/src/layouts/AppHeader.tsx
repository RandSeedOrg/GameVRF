import { theme } from "antd";
import { Header } from "antd/es/layout/layout";
import { AppUser } from "./AppUser";


import { ReactNode } from "react";
import { TimeView } from "./TimeView";

export function AppHeader({ children }: { children: ReactNode }) {
  const {
    token: { colorBgContainer },
  } = theme.useToken();
  
  return (
    <Header style={{ padding: 0, background: colorBgContainer }} className='flex items-center justify-between'>
      {children}
      <TimeView />
      <AppUser />
    </Header>
  );
}