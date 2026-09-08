import { AISecSDKException, ErrorType } from '@cdot65/prisma-airs-sdk';

export type FailureCategory = 'timeout' | 'cancelled' | 'http' | 'network'
  | 'response-validation' | 'request-validation' | 'configuration'
  | 'sdk-error' | 'scanner-error' | 'incomplete-verdict' | 'unknown-error';

export interface ScanFailure {
  event: 'airs_scan_failure';
  category: FailureCategory;
  http_status?: number;
  native_code?: number;
  request_id?: string | number;
}

function correlation(value: unknown): string | number | undefined {
  if (typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 2_147_483_647) return value;
  if (typeof value === 'string' && /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(value)) return value;
  return undefined;
}

export function failureRecord(category: FailureCategory, requestId: unknown): ScanFailure {
  const request_id = correlation(requestId);
  return { event: 'airs_scan_failure', category, ...(request_id === undefined ? {} : { request_id }) };
}

// Read typed SDK metadata only. Never inspect message, stack, cause, response
// bodies or arbitrary object fields: they can contain prompt or bearer material.
export function exceptionRecord(error: unknown, requestId: unknown): ScanFailure {
  if (error instanceof DOMException && (error.name === 'TimeoutError' || error.name === 'AbortError')) {
    const code = error.code;
    return {
      ...failureRecord(error.name === 'TimeoutError' ? 'timeout' : 'cancelled', requestId),
      ...(typeof code === 'number' && Number.isInteger(code) && code >= 0 && code <= 65_535 ? { native_code: code } : {}),
    };
  }
  if (error instanceof AISecSDKException) {
    let category: FailureCategory = 'sdk-error';
    if (error.failureKind === 'http') category = 'http';
    else if (error.failureKind === 'network') category = 'network';
    else if (error.errorType === ErrorType.RESPONSE_VALIDATION) category = 'response-validation';
    else if (error.errorType === ErrorType.USER_REQUEST_PAYLOAD_ERROR) category = 'request-validation';
    else if (error.errorType === ErrorType.MISSING_VARIABLE) category = 'configuration';
    const status = error.statusCode;
    return {
      ...failureRecord(category, requestId),
      ...(typeof status === 'number' && Number.isInteger(status) && status >= 100 && status <= 599 ? { http_status: status } : {}),
    };
  }
  return failureRecord('unknown-error', requestId);
}
