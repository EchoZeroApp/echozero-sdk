/** Structured signal envelope types (#1098). */

export const StructuredSignalEventType = {
  TRADE_IDEA: 'trade_idea',
  BUY: 'buy',
  SCALE_IN: 'scale_in',
  SELL: 'sell',
  PARTIAL_SELL: 'partial_sell',
  AMEND: 'amend',
  BREAKEVEN: 'breakeven',
  CANCEL: 'cancel',
  POSITION_UPDATE: 'position_update',
  TRADE_REVIEW: 'trade_review',
} as const;

export type StructuredSignalEventType =
  (typeof StructuredSignalEventType)[keyof typeof StructuredSignalEventType];

export const StructuredSignalChain = {
  SOLANA: 'solana',
  HYPERLIQUID: 'hyperliquid',
} as const;

export type StructuredSignalChain =
  (typeof StructuredSignalChain)[keyof typeof StructuredSignalChain];

export const StructuredSignalSide = {
  LONG: 'long',
  SHORT: 'short',
} as const;

export type StructuredSignalSide =
  (typeof StructuredSignalSide)[keyof typeof StructuredSignalSide];

export const StructuredSignalTradeType = {
  SPOT: 'spot',
  PERP: 'perp',
  VIRTUAL: 'virtual',
} as const;

export type StructuredSignalTradeType =
  (typeof StructuredSignalTradeType)[keyof typeof StructuredSignalTradeType];

export type InboundSignalBody = Record<string, unknown> & {
  text?: string;
  idempotencyKey?: string;
  eventType?: StructuredSignalEventType;
  chain?: StructuredSignalChain;
  symbol?: string;
  side?: StructuredSignalSide;
  tradeType?: StructuredSignalTradeType;
  amount?: number;
  reasoning?: string;
  confidence?: number;
  positionRef?: string;
  action?: 'buy' | 'sell';
  tokenAddress?: string;
};

export type AgentSignalResponse = {
  outcome?: 'matched' | 'unmatched' | 'skipped' | 'error';
  signalId?: string;
  status?: string;
  skipReason?: string;
};

export type OutboundExecutionWebhook = {
  event: 'signal.execution' | 'signal.execution.failed';
  signalId: string;
  developerAgentId: string;
  status: string;
  executionResult?: {
    success: boolean;
    txHash?: string;
    executedAmountUsd?: number;
    errorMessage?: string;
    executedAt?: string;
  };
  timestamp: string;
};
