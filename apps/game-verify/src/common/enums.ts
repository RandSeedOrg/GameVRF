export enum BatchState {
  New = '0',
  Initialized = '1',
  Running = '2',
  Paused = '3',
  Finished = '4',
}

export enum Crypto {
  ICP = '0',
  USDT = '1',
}

export enum RewardCrypto {
  BONUS = '0',
}

export enum StakingPoolStatus {
  /** 新建 */
  Created = '0',
  /** 开放质押 */
  Open = '1', 
  /** 关闭，不允许质押 */
  Closed = '2', 
  /** 已完结 */
  Finished = '3', 
  /** 已取消 */
  Cancelled = '4',
};


export enum ProductType {
  LuckyNickel = '0',
  QuickQuid = '1',
}

export enum ProposalStatus {
  /** 新建 */
  Created = '0',
  /** 投票中 */
  Voting = '1', 
  /** 已通过 */
  Passed = '2', 
  /** 已拒绝 */
  Rejected = '3', 
  /** 已取消 */
  Executed = '4',
};