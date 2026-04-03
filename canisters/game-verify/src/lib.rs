pub(crate) mod daily4;
pub(crate) mod keno;

use crate::daily4::GenerateDaily4BallRequest;
use crate::daily4::GenerateDaily4BallsRequest;
use crate::daily4::GenerateDaily4BallsResponse;
use crate::keno::GenerateKenoClientRandomNumbersRequest;
use crate::keno::GenerateKenoNumbersRequest;

ic_cdk::export_candid!();