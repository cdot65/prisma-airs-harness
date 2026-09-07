// These are additional restrictions on the original JWT. Native AIRS still
// verifies its signature before opening an MCP connection or executing tools.
export function mcpRequest(req, claims, policy) {
  const mcp = policy.mcp;
  if (!mcp || req.url !== mcp.path || !['POST', 'GET', 'DELETE'].includes(req.method)) {
    return [403, 'MCP resource is not authorized for Terminal'];
  }
  const audience = Array.isArray(claims.aud) ? claims.aud : [claims.aud];
  if (claims.azp !== mcp.client || claims.iss !== mcp.issuer ||
      claims.portkey_oid !== mcp.organisation || claims.portkey_workspace !== policy.workspace ||
      audience.length !== 1 || audience[0] !== mcp.audience ||
      !Array.isArray(claims.airs_roles) || !claims.airs_roles.includes(mcp.role) ||
      typeof claims.scope !== 'string' || !claims.scope.split(/\s+/).includes('portkey.mcp.invoke')) {
    return [403, 'MCP client, audience, workspace, scope and scanner role are required'];
  }
  if (req.method !== 'POST' && (req.headers['transfer-encoding'] || Number(req.headers['content-length'] ?? 0) !== 0)) {
    return [400, 'MCP control requests must not contain a body'];
  }
  return null;
}

export function mcpBody(parsed, policy) {
  const methods = ['initialize', 'notifications/initialized', 'ping', 'tools/list', 'tools/call', 'notifications/cancelled'];
  return parsed.jsonrpc === '2.0' && methods.includes(parsed.method) &&
    (parsed.method !== 'tools/call' || policy.mcp.tools.includes(parsed.params?.name));
}
