import { Button, Result } from "antd";
import { useNavigate } from "react-router-dom";

export function PermissionDenied() {
  const navigate = useNavigate();

  const handleGoHome = () => {
    navigate('/')
  };

  return (
    <div className='flex justify-center items-center'>
      <Result
        status="403"
        title="403"
        subTitle="Sorry, you are not authorized to access this page."
        extra={<Button onClick={handleGoHome} type="primary">Back Home</Button>}
      />
    </div>
  );
}