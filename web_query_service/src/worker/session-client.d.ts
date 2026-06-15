// 手写类型声明：供前端 TS（无 allowJs）import 同目录 `session-client.js`。

export interface SessionSnapshot {
  token: string;
  sk: string;
  sid: string;
  exp: number;
  quota: number;
}

export interface SessionManagerDeps {
  getChallenge: () => Promise<{ seed: string; difficulty: number }>;
  solvePow: (seed: string, difficulty: number) => Promise<string>;
  postSession: (seed: string, nonce: string) => Promise<{ token: string; sk: string; exp: number; quota: number }>;
  signQuery: (path: string, params: Record<string, string>, snapshot: SessionSnapshot) => Promise<string>;
  doFetch: (url: string) => Promise<{ status: number; json?: () => any }>;
  now: () => number;
  skewMs?: number;
}

export interface QueryResult {
  status: number;
  json?: () => any;
  retried: boolean;
}

export interface SessionManager {
  getSession: () => Promise<SessionSnapshot>;
  invalidate: (snapshot: SessionSnapshot | null) => void;
  requestQuery: (path: string, params?: Record<string, string>) => Promise<QueryResult>;
  _peek: () => SessionSnapshot | null;
}

export function parseSid(token: string): string;
export function createSessionManager(deps: SessionManagerDeps): SessionManager;
