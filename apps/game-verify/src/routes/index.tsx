import AdminLayout from "@/layouts";
import { Navigate, Outlet, Route } from "react-router-dom";

export function PermissionRoutes() {

  return (
    <Route path="/" element={<AdminLayout />}>
      
    </Route>
  )
}


export function NoPermissionRoutes() {
  return (
    <>
      <Route path="*" element={<Navigate to="/login" replace />} />
    </>
  );
}