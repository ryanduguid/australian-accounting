// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

// What a conforming client sends and how the harness and the live check read
// replies. No dependency, so the live check runs with Node alone.
export const MODERN = '2026-07-28';
export const LEGACY = '2025-11-25';

export const meta = { 'io.modelcontextprotocol/protocolVersion': MODERN, 'io.modelcontextprotocol/clientCapabilities': {} };

// The headers a conforming client sends with `message` in `era`.
export function headers(message, era, extra = {}) {
  const sent = { 'content-type': 'application/json', accept: 'application/json, text/event-stream', ...extra };
  if (era === MODERN) {
    sent['mcp-protocol-version'] = MODERN;
    sent['mcp-method'] = message.method;
    if (message.method === 'tools/call') sent['mcp-name'] = message.params.name;
  } else if (era === LEGACY && message.method !== 'initialize') {
    sent['mcp-protocol-version'] = LEGACY;
  }
  return sent;
}

export function message(era, id, method, params = {}) {
  return { jsonrpc: '2.0', id, method, params: era === MODERN ? { ...params, _meta: meta } : params };
}

// The text of `result` in a body `{"jsonrpc":"2.0","id":<id>,"result":<result>}`.
export function resultText(body, id) {
  const head = `{"jsonrpc":"2.0","id":${JSON.stringify(id)},"result":`;
  if (!body.startsWith(head) || !body.endsWith('}')) throw new Error(`unexpected reply: ${body.slice(0, 300)}`);
  return body.slice(head.length, -1);
}

// The raw text of each element of a JSON array's text.
export function elements(array) {
  const parts = [];
  let depth = 0;
  let start = 1;
  let quoted = false;
  for (let i = 0; i < array.length; i += 1) {
    const c = array[i];
    if (quoted) {
      if (c === '\\') i += 1;
      else if (c === '"') quoted = false;
    } else if (c === '"') quoted = true;
    else if (c === '[' || c === '{') depth += 1;
    else if (c === ']' || c === '}') {
      depth -= 1;
      if (depth === 0) parts.push(array.slice(start, i));
    } else if (c === ',' && depth === 1) {
      parts.push(array.slice(start, i));
      start = i + 1;
    }
  }
  return parts.filter(part => part.length > 0);
}

// The `_meta` text of a 2026-07-28 tools/list result, which names the server.
export function metaOf(listResult) {
  const match = listResult.match(/^\{"resultType":"complete","_meta":(.*?),"ttlMs":/);
  if (!match) throw new Error(`unexpected tools/list result: ${listResult.slice(0, 200)}`);
  return match[1];
}

// A native 2026-07-28 tool result as a server whose `_meta` text is `mine`
// gives it: the same bytes but the closing `_meta`, which names the server.
export function servedBy(native, nativeMeta, mine) {
  const suffix = `,"_meta":${nativeMeta}}`;
  if (!native.startsWith('{"resultType":"complete",') || !native.endsWith(suffix)) {
    throw new Error(`unexpected native result: ${native.slice(0, 200)}`);
  }
  return `${native.slice(0, -suffix.length)},"_meta":${mine}}`;
}
